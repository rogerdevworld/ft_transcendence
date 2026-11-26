use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    features::catalog::dto::UserRole,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i32,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i32,
    pub role: UserRole,
}

pub fn session_redis_key(token: &str) -> String {
    format!("auth:session:{}", token)
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or(AppError::Unauthorized)?;

        let mut conn = state.redis.clone();
        let raw_json: Option<String> = conn
            .get(session_redis_key(token))
            .await
            .map_err(|e| AppError::Internal(format!("Error leyendo sesión en Redis: {}", e)))?;

        let json_str = raw_json.ok_or(AppError::Unauthorized)?;
        let session: SessionData = serde_json::from_str(&json_str)
            .map_err(|e| AppError::Internal(format!("Sesión corrupta en Redis: {}", e)))?;

        Ok(CurrentUser {
            id: session.user_id,
            role: session.role,
        })
    }
}

pub fn ensure_creator_or_admin(user: &CurrentUser) -> Result<(), AppError> {
    match user.role {
        UserRole::Artist | UserRole::Creator | UserRole::Admin => Ok(()),
        UserRole::Listener => Err(AppError::Forbidden(
            "Se requiere rol ARTIST o ADMIN para esta acción".to_string(),
        )),
    }
}

pub fn ensure_owner_or_admin(user: &CurrentUser, owner_id: Option<i32>) -> Result<(), AppError> {
    if matches!(user.role, UserRole::Admin) {
        return Ok(());
    }

    match owner_id {
        Some(oid) if oid == user.id => Ok(()),
        _ => Err(AppError::Forbidden(
            "No tienes permisos sobre este recurso".to_string(),
        )),
    }
}