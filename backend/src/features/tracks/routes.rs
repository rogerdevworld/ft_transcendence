use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    state::AppState,
    features::tracks::handlers::{create, get_by_id, list, update, delete as delete_track},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tracks", get(list))
        .route("/tracks", post(create))
        .route("/tracks/{id}", get(get_by_id))
        .route("/tracks/{id}", patch(update))
        .route("/tracks/{id}", delete(delete_track))
}