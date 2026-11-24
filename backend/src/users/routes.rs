use axum::{
    routing::{get, patch},
    Router,
};

use crate::{
    state::AppState,
    users::handlers::{create, delete, get_by_id, list, update},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list).post(create))
        .route(
            "/users/{id}",
            get(get_by_id).patch(update).delete(delete),
        )
}