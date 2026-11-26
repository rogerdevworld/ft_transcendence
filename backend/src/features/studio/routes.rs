use axum::{routing::get, Router};

use crate::{features::studio::handlers::list_releases, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/me/studio/releases", get(list_releases))
}