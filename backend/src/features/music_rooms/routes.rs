use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{
    features::music_rooms::handlers::{
        create, delete, enqueue_track_handler, get_by_id, list, next_track_handler, room_ws_handler, update, vote_track_handler,
    }, state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rooms", get(list).post(create))
        .route("/rooms/{id}", get(get_by_id).patch(update).delete(delete))
        .route("/rooms/{id}/ws", get(room_ws_handler))
        .route("/rooms/{id}/queue", post(enqueue_track_handler))
        .route("/rooms/{id}/queue/{track_id}/vote", post(vote_track_handler))
        .route("/rooms/{id}/next", post(next_track_handler))
}