use axum::{
    extract::DefaultBodyLimit,
    routing::post,
    Router,
};

use crate::{
    features::uploads::handlers::{upload_image, upload_media},
    state::AppState,
};

const BODY_LIMIT_BYTES: usize = 520 * 1024 * 1024; // 520 MB con margen para multipart

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/uploads/images", post(upload_image))
        .route("/uploads/media", post(upload_media))
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
}