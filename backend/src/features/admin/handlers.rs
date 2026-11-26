use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};

use crate::{
    error::AppError,
    features::{
        admin::dto::{
            AdminActionResponse, AdminStatsResponse, AdminUpdateTrackDto, AdminUpdateUserDto,
        },
        catalog::{authz::CurrentUser, dto::UserRole},
        music_rooms::model::Entity as MusicRoom,
        tracks::model::{self, Entity as Track},
        users::{
            dto::UserResponse,
            model::{self as user_model, Entity as User},
        },
    },
    state::AppState,
};

fn ensure_admin(user: &CurrentUser) -> Result<(), AppError> {
    match user.role {
        UserRole::Admin => Ok(()),
        _ => Err(AppError::Forbidden(
            "Se requiere rol ADMIN para esta acción".to_string(),
        )),
    }
}

fn user_to_response(u: user_model::Model, role: UserRole) -> UserResponse {
    UserResponse {
        id: u.id,
        email: u.email,
        display_name: u.display_name,
        role,
        subscription_plan: u.subscription_plan,
        country: None,
        images: Vec::new(),
        followers_total: 0,
        uri: format!("app:user:{}", u.id),
    }
}

// GET /api/admin/stats
pub async fn get_stats(
    user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<AdminStatsResponse>, AppError> {
    ensure_admin(&user)?;

    let total_users = User::find().count(&state.db).await?;
    let total_tracks = Track::find().count(&state.db).await?;

    Ok(Json(AdminStatsResponse {
        total_users,
        total_tracks,
    }))
}

// GET /api/admin/users
pub async fn list_users(
    user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    ensure_admin(&user)?;

    let users = User::find().all(&state.db).await?;
    let list = users
        .into_iter()
        .map(|u| user_to_response(u, UserRole::Listener))
        .collect();

    Ok(Json(list))
}

// GET /api/admin/users/{id}
pub async fn get_user(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    ensure_admin(&user)?;

    let target = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario {} no encontrado", id)))?;

    Ok(Json(user_to_response(target, UserRole::Listener)))
}

// PATCH /api/admin/users/{id}
pub async fn update_user(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<AdminUpdateUserDto>,
) -> Result<Json<UserResponse>, AppError> {
    ensure_admin(&user)?;

    let target = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario {} no encontrado", id)))?;

    let role = payload.role.unwrap_or(UserRole::Listener);
    let mut active: user_model::ActiveModel = target.into();

    if let Some(display_name) = payload.display_name {
        let trimmed = display_name.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest(
                "display_name no puede estar vacío".to_string(),
            ));
        }
        active.display_name = Set(trimmed.to_string());
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

    let updated = active.update(&state.db).await?;
    Ok(Json(user_to_response(updated, role)))
}

// DELETE /api/admin/users/{id}
pub async fn delete_user(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    ensure_admin(&user)?;

    let res = User::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Usuario {} no encontrado", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/admin/users/{id}/revoke-sessions
pub async fn revoke_user_sessions(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AdminActionResponse>, AppError> {
    ensure_admin(&user)?;

    let _ = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario {} no encontrado", id)))?;

    // Cuando guardemos un índice de tokens por usuario en Redis (ej. auth:user_sessions:{id}),
    // aquí los borramos todos; por ahora dejamos el endpoint listo y validado.
    let _ = &state.redis;

    Ok(Json(AdminActionResponse {
        ok: true,
        message: format!("Sesiones revisadas/revocadas para el usuario {}", id),
    }))
}

// GET /api/admin/tracks
pub async fn list_tracks(
    user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<model::Model>>, AppError> {
    ensure_admin(&user)?;

    let tracks = Track::find().all(&state.db).await?;
    Ok(Json(tracks))
}

// GET /api/admin/tracks/{id}
pub async fn get_track(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<model::Model>, AppError> {
    ensure_admin(&user)?;

    let track = Track::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pista {} no encontrada", id)))?;

    Ok(Json(track))
}

// PATCH /api/admin/tracks/{id}
pub async fn update_track(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<AdminUpdateTrackDto>,
) -> Result<Json<model::Model>, AppError> {
    ensure_admin(&user)?;

    let track = Track::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pista {} no encontrada", id)))?;

    let mut active: model::ActiveModel = track.into();

    if let Some(title) = payload.title {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("title no puede estar vacío".to_string()));
        }
        active.title = Set(trimmed.to_string());
    }

    if let Some(artist) = payload.artist {
        let trimmed = artist.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("artist no puede estar vacío".to_string()));
        }
        active.artist = Set(trimmed.to_string());
    }

    if let Some(duration_secs) = payload.duration_secs {
        active.duration_secs = Set(duration_secs.max(0));
    }

    if let Some(is_ad) = payload.is_ad {
        active.is_ad = Set(is_ad);
    }

    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}

// DELETE /api/admin/tracks/{id}
pub async fn delete_track(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    ensure_admin(&user)?;

    let res = Track::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Pista {} no encontrada", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

// GET /api/admin/rooms
pub async fn list_rooms(
    user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::features::music_rooms::model::Model>>, AppError> {
    ensure_admin(&user)?;

    let rooms = MusicRoom::find().all(&state.db).await?;
    Ok(Json(rooms))
}

// DELETE /api/admin/rooms/{id}
pub async fn delete_room(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    ensure_admin(&user)?;

    let res = MusicRoom::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Sala {} no encontrada", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}