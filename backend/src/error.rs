use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    #[error("Petición inválida: {0}")]
    BadRequest(String),

    #[error("No autorizado")]
    Unauthorized,

    #[error("Error de base de datos: {0}")]
    Database(#[from] DbErr),

    #[error("Error interno del servidor")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                String::from("no authorization provided"),
            ),
            AppError::Database(db_err) => {
                tracing::error!("Error de base de datos: {:?}", db_err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    String::from("Error al consultar la base de datos"),
                )
            }
            AppError::Internal(detalle) => {
                tracing::error!("Error interno: {}", detalle);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    String::from("occurred an internal error"),
                )
            }
        };

        let body = Json(ErrorBody {
            error: error_code.to_string(),
            message,
        });

        (status, body).into_response()
    }
}