use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    state::AppState,
    features::tracks::handlers::{create, get_by_id, list},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tracks", get(list).post(create))
        .route("/tracks/{id}", get(get_by_id))
}