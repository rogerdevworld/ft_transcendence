use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::{
    error::AppError,
    features::{
        catalog::dto::{parse_images, serialize_images, ImageObject, UserRole},
        users::{
            dto::{CreateUserDto, UpdateUserDto, UserResponse},
            model::{self, Entity as User},
        },
    },
    state::AppState,
};

fn role_to_db(role: &UserRole) -> String {
    match role {
        UserRole::Listener => "listener".to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn role_from_db(raw: &str) -> UserRole {
    match raw.trim().to_lowercase().as_str() {
        "listener" | "user" => UserRole::Listener,
        "admin" => UserRole::Admin,
        "creator" | "artist" => UserRole::Creator,
        _ => UserRole::Listener,
    }
}

fn to_response(user: model::Model) -> UserResponse {
    let role = role_from_db(&user.role);
    let images = parse_images(&user.images);

    UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        role,
        subscription_plan: user.subscription_plan,
        country: user.country,
        images,
        followers_total: 0,
        uri: format!("app:user:{}", user.id),
    }
}

// POST /api/users
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let email = payload.email.trim().to_lowercase();
    let display_name = payload.display_name.trim().to_string();
    let password = payload.password.trim().to_string();

    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("El email no es válido".to_string()));
    }
    if password.is_empty() {
        return Err(AppError::BadRequest(
            "La contraseña (password) es obligatoria".to_string(),
        ));
    }
    if display_name.is_empty() {
        return Err(AppError::BadRequest(
            "El nombre visible (display_name) es obligatorio".to_string(),
        ));
    }

    let role = payload.role.unwrap_or(UserRole::Listener);
    let plan = payload
        .subscription_plan
        .map(|p| p.trim().to_uppercase())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "FREE".to_string());
    let country = payload
        .country
        .map(|c| c.trim().to_uppercase())
        .filter(|c| !c.is_empty());
    let images_json = serialize_images(&payload.images);

    let new_user = model::ActiveModel {
        email: Set(email),
        password_hash: Set(Some(password)),
        display_name: Set(display_name),
        role: Set(role_to_db(&role)),
        subscription_plan: Set(plan),
        country: Set(country),
        images: Set(images_json),
        ..Default::default()
    };

    let created = new_user.insert(&state.db).await?;
    Ok((StatusCode::CREATED, Json(to_response(created))))
}

// GET /api/users
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = User::find().all(&state.db).await?;
    let items = users.into_iter().map(to_response).collect();
    Ok(Json(items))
}

// GET /api/users/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let found = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario {} no encontrado", id)))?;

    Ok(Json(to_response(found)))
}

// PATCH /api/users/{id}
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<UserResponse>, AppError> {
    let existing = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario {} no encontrado", id)))?;

    let mut active: model::ActiveModel = existing.into();

    if let Some(display_name) = payload.display_name {
        let trimmed = display_name.trim().to_string();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest(
                "display_name no puede estar vacío".to_string(),
            ));
        }
        active.display_name = Set(trimmed);
    }

    if let Some(plan) = payload.subscription_plan {
        let normalized = plan.trim().to_uppercase();
        if normalized.is_empty() {
            return Err(AppError::BadRequest(
                "subscription_plan no puede estar vacío".to_string(),
            ));
        }
        active.subscription_plan = Set(normalized);
    }

    if let Some(country) = payload.country {
        let normalized = country.trim().to_uppercase();
        active.country = Set(if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        });
    }

    if let Some(images) = payload.images {
        active.images = Set(serialize_images(&Some(images)));
    }

    let updated = active.update(&state.db).await?;
    Ok(Json(to_response(updated)))
}

// DELETE /api/users/{id}
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let res = User::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Usuario {} no encontrado", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}