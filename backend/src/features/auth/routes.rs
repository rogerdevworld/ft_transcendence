use axum::{routing::post, Router};

use crate::{
    features::auth::handlers::{login, verify_2fa},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/verify-2fa", post(verify_2fa))
}