use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::EntityTrait;

use crate::{
    error::AppError,
    features::{
        music_rooms::model::Entity as MusicRoom,
        webrtc_signaling::{
            dto::{SignalMessage, SignalScope, SignalingInfoResponse},
            hub::SignalHub,
        },
    },
    state::AppState,
};

fn extract_scope_and_room(msg: &SignalMessage) -> (SignalScope, Option<i64>) {
    match msg {
        SignalMessage::PeerAnnounce { scope, room_id, .. }
        | SignalMessage::PeerLeave { scope, room_id, .. }
        | SignalMessage::Offer { scope, room_id, .. }
        | SignalMessage::Answer { scope, room_id, .. }
        | SignalMessage::IceCandidate { scope, room_id, .. } => (scope.clone(), *room_id),
    }
}

fn dispatch_general_signal(signal_hub: &SignalHub, msg: &SignalMessage) -> Result<(), AppError> {
    match msg {
        SignalMessage::PeerAnnounce {
            scope,
            resource_id,
            peer_id,
            ..
        }
        | SignalMessage::PeerLeave {
            scope,
            resource_id,
            peer_id,
            ..
        } => match scope {
            SignalScope::Swarm => {
                let res = resource_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "resource_id es obligatorio en scope 'swarm'".to_string(),
                        )
                    })?;
                signal_hub.publish(&format!("swarm:{}", res), msg);
                Ok(())
            }
            SignalScope::Direct => {
                if peer_id.trim().is_empty() {
                    return Err(AppError::BadRequest(
                        "peer_id es obligatorio en scope 'direct'".to_string(),
                    ));
                }
                signal_hub.publish(&format!("peer:{}", peer_id.trim()), msg);
                Ok(())
            }
            SignalScope::Room => Ok(()),
        },
        SignalMessage::Offer {
            scope,
            resource_id,
            to_peer_id,
            ..
        }
        | SignalMessage::Answer {
            scope,
            resource_id,
            to_peer_id,
            ..
        }
        | SignalMessage::IceCandidate {
            scope,
            resource_id,
            to_peer_id,
            ..
        } => match scope {
            SignalScope::Direct => {
                let target = to_peer_id.trim();
                if target.is_empty() {
                    return Err(AppError::BadRequest(
                        "to_peer_id es obligatorio en scope 'direct'".to_string(),
                    ));
                }
                signal_hub.publish(&format!("peer:{}", target), msg);
                Ok(())
            }
            SignalScope::Swarm => {
                let res = resource_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "resource_id es obligatorio en scope 'swarm'".to_string(),
                        )
                    })?;
                signal_hub.publish(&format!("swarm:{}", res), msg);
                Ok(())
            }
            SignalScope::Room => Ok(()),
        },
    }
}

// GET /api/webrtc/info
pub async fn get_general_info() -> Json<SignalingInfoResponse> {
    Json(SignalingInfoResponse {
        room_id: None,
        signaling_ready: true,
        transport: "room-hub + swarm-direct-hub".to_string(),
    })
}

// POST /api/webrtc/signal (señalización general: swarm / direct / room)
pub async fn post_general_signal(
    State(state): State<AppState>,
    Json(payload): Json<SignalMessage>,
) -> Result<Json<SignalMessage>, AppError> {
    let (scope, room_id) = extract_scope_and_room(&payload);

    match scope {
        SignalScope::Room => {
            let rid = room_id.ok_or_else(|| {
                AppError::BadRequest("room_id es obligatorio cuando scope es 'room'".to_string())
            })?;
            let rid_i32 = i32::try_from(rid)
                .map_err(|_| AppError::BadRequest("room_id fuera de rango".to_string()))?;
            let _ = MusicRoom::find_by_id(rid_i32)
                .one(&state.db)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Sala {} no encontrada", rid)))?;

            state.ws_hub.send_webrtc_signal(rid, payload.clone());
        }
        SignalScope::Swarm | SignalScope::Direct => {
            // Si aún no añadiste `signal_hub: SignalHub` a `AppState`, puedes cambiarlo a `state.signal_hub`
            let fallback_hub = SignalHub::new();
            dispatch_general_signal(&fallback_hub, &payload)?;
        }
    }

    Ok(Json(payload))
}

// GET /api/rooms/{room_id}/webrtc/info
pub async fn get_room_info(
    State(state): State<AppState>,
    Path(room_id): Path<i64>,
) -> Result<Json<SignalingInfoResponse>, AppError> {
    let rid_i32 = i32::try_from(room_id)
        .map_err(|_| AppError::BadRequest("room_id fuera de rango".to_string()))?;

    let _ = MusicRoom::find_by_id(rid_i32)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Sala {} no encontrada", room_id)))?;

    Ok(Json(SignalingInfoResponse {
        room_id: Some(room_id),
        signaling_ready: true,
        transport: "websocket-room-hub".to_string(),
    }))
}

// POST /api/rooms/{room_id}/webrtc/signal
pub async fn post_room_signal(
    State(state): State<AppState>,
    Path(room_id): Path<i64>,
    Json(payload): Json<SignalMessage>,
) -> Result<Json<SignalMessage>, AppError> {
    let rid_i32 = i32::try_from(room_id)
        .map_err(|_| AppError::BadRequest("room_id fuera de rango".to_string()))?;

    let _ = MusicRoom::find_by_id(rid_i32)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Sala {} no encontrada", room_id)))?;

    let (scope, msg_room_id) = extract_scope_and_room(&payload);
    if !matches!(scope, SignalScope::Room) || msg_room_id != Some(room_id) {
        return Err(AppError::BadRequest(
            "El mensaje debe tener scope 'room' y room_id coincidente con la ruta".to_string(),
        ));
    }

    state.ws_hub.send_webrtc_signal(room_id, payload.clone());

    Ok(Json(payload))
}