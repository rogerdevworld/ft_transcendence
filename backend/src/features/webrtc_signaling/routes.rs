use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    features::webrtc_signaling::handlers::{
        get_general_info, get_room_info, post_general_signal, post_room_signal,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/webrtc/info", get(get_general_info))
        .route("/webrtc/signal", post(post_general_signal))
        .route("/rooms/{room_id}/webrtc/info", get(get_room_info))
        .route("/rooms/{room_id}/webrtc/signal", post(post_room_signal))
}