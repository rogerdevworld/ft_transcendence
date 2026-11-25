use axum::{routing::get, Router};

use crate::{
    features::media_delivery::handlers::{get_chunk, stream_range},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tracks/{id}/chunks/{index}", get(get_chunk))
        .route("/tracks/{id}/stream", get(stream_range))
}