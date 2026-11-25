use axum::{
    Json,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};

use crate::{
    error::AppError,
    features::{
        music_rooms::{
            self, RoomWsEvent,
            dto::{
                CreateRoomDto, EnqueueTrackRequest, QueuedTrack, RoomResponse, UpdateRoomDto,
                VoteResponse, VoteTrackRequest, 
            },
            model::{self, normalize_visibility, normalize_vote_license, Entity as MusicRoom},
        },
        users::model::Entity as User,
    },
    state::AppState,
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
            "The title cannot be empty".to_string(),
        ));
    }

    let _owner = User::find_by_id(payload.owner_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("User with id {} not found", payload.owner_id))
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

// POST /api/rooms/{id}/queue
pub async fn enqueue_track_handler(
    Path(room_id): Path<i64>,
    State(mut state): State<AppState>,
    Json(body): Json<EnqueueTrackRequest>,
) -> Result<(StatusCode, Json<QueuedTrack>), (StatusCode, String)> {
    let (was_added, votes) = music_rooms::enqueue_track_in_redis(&mut state.redis, room_id, body.track_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if was_added {
        state.ws_hub.publish(
            room_id,
            RoomWsEvent::TrackQueued {
                room_id,
                track_id: body.track_id,
                votes,
            },
        );
        return Ok((
            StatusCode::CREATED,
            Json(QueuedTrack {
                track_id: body.track_id,
                votes,
            }),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(QueuedTrack {
            track_id: body.track_id,
            votes,
        }),
    ))
}

// POST /api/rooms/{id}/queue/{track_id}/vote
pub async fn vote_track_handler(
    Path((room_id, track_id)): Path<(i64, i64)>,
    State(mut state): State<AppState>,
    Json(body): Json<VoteTrackRequest>,
) -> Result<Json<VoteResponse>, (StatusCode, String)> {
    let (voted, votes) =
        music_rooms::toggle_vote_in_redis(&mut state.redis, room_id, track_id, body.user_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.ws_hub.publish(
        room_id,
        RoomWsEvent::VoteUpdated {
            room_id,
            track_id,
            user_id: body.user_id,
            voted,
            votes,
        },
    );

    Ok(Json(VoteResponse {
        track_id,
        voted,
        votes,
    }))
}

// GET /api/rooms/{id}/ws
pub async fn room_ws_handler(
    Path(room_id): Path<i64>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: i64, mut state: AppState) {
    let mut rx = state.ws_hub.subscribe(room_id);
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let current_queue = music_rooms::get_queue(&mut state.redis, room_id)
        .await
        .unwrap_or_default();
    let snapshot = RoomWsEvent::RoomSnapshot {
        room_id,
        queue: current_queue,
    };

    if let Ok(json) = serde_json::to_string(&snapshot) {
        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    state.ws_hub.publish(room_id, RoomWsEvent::MemberJoined { room_id });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!(room_id, %text, "Mensaje recibido por WS");
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    state.ws_hub.publish(room_id, RoomWsEvent::MemberLeft { room_id });
}