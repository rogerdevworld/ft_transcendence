use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::{
    error::AppError,
    music_rooms::{
        dto::{CreateRoomDto, RoomResponse, UpdateRoomDto},
        model::{
            self, normalize_visibility, normalize_vote_license, Entity as MusicRoom,
        },
    },
    state::AppState,
    users::model::Entity as User,
};

fn to_response(room: model::Model) -> RoomResponse {
    RoomResponse {
        id: room.id,
        owner_id: room.owner_id,
        title: room.title,
        visibility: room.visibility,
        vote_license: room.vote_license,
    }
}

// POST /api/rooms
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateRoomDto>,
) -> Result<(StatusCode, Json<RoomResponse>), AppError> {
    let title = payload.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::BadRequest(
            "El título de la sala (title) es obligatorio".to_string(),
        ));
    }

    let _owner = User::find_by_id(payload.owner_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Usuario propietario (owner_id {}) no encontrado",
                payload.owner_id
            ))
        })?;

    let new_room = model::ActiveModel {
        owner_id: Set(payload.owner_id),
        title: Set(title),
        visibility: Set(normalize_visibility(payload.visibility)),
        vote_license: Set(normalize_vote_license(payload.vote_license)),
        ..Default::default()
    };

    let created = new_room.insert(&state.db).await?;
    Ok((StatusCode::CREATED, Json(to_response(created))))
}

// GET /api/rooms
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<RoomResponse>>, AppError> {
    let rooms = MusicRoom::find().all(&state.db).await?;
    Ok(Json(rooms.into_iter().map(to_response).collect()))
}

// GET /api/rooms/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<RoomResponse>, AppError> {
    let found = MusicRoom::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Sala {} no encontrada", id)))?;

    Ok(Json(to_response(found)))
}

// PATCH /api/rooms/{id}
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateRoomDto>,
) -> Result<Json<RoomResponse>, AppError> {
    let existing = MusicRoom::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Sala {} no encontrada", id)))?;

    let mut active: model::ActiveModel = existing.into();

    if let Some(title) = payload.title {
        let trimmed = title.trim().to_string();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest(
                "title no puede estar vacío".to_string(),
            ));
        }
        active.title = Set(trimmed);
    }

    if let Some(visibility) = payload.visibility {
        active.visibility = Set(normalize_visibility(Some(visibility)));
    }

    if let Some(vote_license) = payload.vote_license {
        active.vote_license = Set(normalize_vote_license(Some(vote_license)));
    }

    let updated = active.update(&state.db).await?;
    Ok(Json(to_response(updated)))
}

// DELETE /api/rooms/{id}
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let res = MusicRoom::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Sala {} no encontrada", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}