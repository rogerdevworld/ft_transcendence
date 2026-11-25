use axum::{
    routing::{get, patch},
    Router,
};

use crate::{
    music_rooms::handlers::{create, delete, get_by_id, list, update},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rooms", get(list).post(create))
        .route(
            "/rooms/{id}",
            get(get_by_id).patch(update).delete(delete),
        )
}