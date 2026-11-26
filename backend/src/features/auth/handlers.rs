use axum::{extract::State, http::StatusCode, Json};
use rand::Rng;
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    features::{
        catalog::{
            authz::{session_redis_key, SessionData},
            dto::UserRole,
        },
        auth::{
            dto::{AuthTokenResponse, LoginChallengeResponse, LoginDto, Verify2FaDto},
        },
        users::{
            dto::UserResponse,
            model::{self, Entity as User},
        },
    },
    state::AppState,
};

const OTP_TTL_SECS: u64 = 300; // 5 minutos
const SESSION_TTL_SECS: u64 = 86_400; // 24 horas
const MAX_OTP_ATTEMPTS: u8 = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorChallenge {
    user_id: i32,
    role: UserRole,
    code: String,
    attempts: u8,
}

fn otp_redis_key(challenge_id: &str) -> String {
    format!("auth:2fa:{}", challenge_id)
}

fn generate_6_digit_code() -> String {
    let num: u32 = rand::random::<u32>() % 1_000_000;
    format!("{:06}", num)
}

// POST /api/auth/login -> Paso 1: valida contraseña y emite challenge 2FA
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> Result<(StatusCode, Json<LoginChallengeResponse>), AppError> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password.trim();

    if email.is_empty() || password.is_empty() {
        return Err(AppError::BadRequest(
            "email y password son obligatorios".to_string(),
        ));
    }

    let user = User::find()
        .filter(model::Column::Email.eq(email))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let valid = user.password_hash.as_deref() == Some(password);
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let role = UserRole::Listener;
    let challenge_id = Uuid::new_v4().to_string();
    let code = generate_6_digit_code();

    // En desarrollo lo registramos en log; en producción aquí llamas a tu servicio de email/SMS
    tracing::info!(user_id = user.id, challenge_id = %challenge_id, code = %code, "Código 2FA generado");

    let challenge: TwoFactorChallenge = TwoFactorChallenge {
        user_id: user.id,
        role,
        code,
        attempts: 0,
    };

    let serialized = serde_json::to_string(&challenge)
        .map_err(|e| AppError::Internal(format!("Error serializando 2FA: {}", e)))?;

    let mut conn = state.redis.clone();
    let _: () = conn
        .set_ex(otp_redis_key(&challenge_id), serialized, OTP_TTL_SECS)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando 2FA en Redis: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(LoginChallengeResponse {
            requires_2fa: true,
            challenge_id,
            expires_in: OTP_TTL_SECS,
        }),
    ))
}

// POST /api/auth/verify-2fa -> Paso 2: verifica OTP y entrega Bearer token
pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(payload): Json<Verify2FaDto>,
) -> Result<Json<AuthTokenResponse>, AppError> {
    let challenge_id = payload.challenge_id.trim();
    let code = payload.code.trim();

    if challenge_id.is_empty() || code.is_empty() {
        return Err(AppError::BadRequest(
            "challenge_id y code son obligatorios".to_string(),
        ));
    }

    let key = otp_redis_key(challenge_id);
    let mut conn = state.redis.clone();

    let raw: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| AppError::Internal(format!("Error leyendo 2FA en Redis: {}", e)))?;

    let raw_str = raw.ok_or(AppError::Unauthorized)?;
    let mut challenge: TwoFactorChallenge = serde_json::from_str(&raw_str)
        .map_err(|e| AppError::Internal(format!("Challenge 2FA corrupto: {}", e)))?;

    if challenge.attempts >= MAX_OTP_ATTEMPTS {
        let _: () = conn.del(&key).await.unwrap_or(());
        return Err(AppError::Unauthorized);
    }

    if challenge.code != code {
        challenge.attempts += 1;
        let updated = serde_json::to_string(&challenge)
            .map_err(|e| AppError::Internal(format!("Error serializando 2FA: {}", e)))?;
        let _: () = conn
            .set_ex(&key, updated, OTP_TTL_SECS)
            .await
            .unwrap_or(());
        return Err(AppError::Unauthorized);
    }

    // Código válido: consumimos el challenge de un solo uso
    let _: () = conn.del(&key).await.unwrap_or(());

    let user = User::find_by_id(challenge.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let token = Uuid::new_v4().to_string();
    let session = SessionData {
        user_id: user.id,
        role: challenge.role.clone(),
    };

    let session_json = serde_json::to_string(&session)
        .map_err(|e| AppError::Internal(format!("Error serializando sesión: {}", e)))?;

    let _: () = conn
        .set_ex(session_redis_key(&token), session_json, SESSION_TTL_SECS)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando sesión en Redis: {}", e)))?;

    Ok(Json(AuthTokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECS,
        user: UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: challenge.role,
            subscription_plan: user.subscription_plan,
            country: None,
            images: Vec::new(),
            followers_total: 0,
            uri: format!("app:user:{}", user.id),
        },
    }))
}