// backend/src/routes/health.rs
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    environment: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/error-demo", get(error_demo))
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: String::from("ok"),
        environment: state.config.app_env.clone(),
    })
}

async fn error_demo() -> Result<Json<HealthResponse>, AppError> {
    Err(AppError::NotFound(String::from("Example room not found")))
}