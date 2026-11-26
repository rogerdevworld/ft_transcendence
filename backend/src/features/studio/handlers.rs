use axum::{extract::State, Json};
use sea_orm::EntityTrait;

use crate::{
    error::AppError,
    features::{
        catalog::dto::{MediaKind, PublishStatus, SourceKind},
        studio::dto::StudioReleaseDto,
        tracks::model::Entity as Track,
    },
    state::AppState,
};

fn parse_media_kind(raw: &str) -> MediaKind {
    match raw.trim().to_ascii_uppercase().as_str() {
        "VIDEO" => MediaKind::Video,
        "PODCAST" => MediaKind::Podcast,
        "AUDIOBOOK" => MediaKind::Audiobook,
        _ => MediaKind::Audio,
    }
}

fn parse_source_kind(raw: &str) -> SourceKind {
    match raw.trim().to_ascii_uppercase().as_str() {
        "EXTERNAL" => SourceKind::External,
        "P2P" => SourceKind::P2p,
        _ => SourceKind::Local,
    }
}

// GET /api/me/studio/releases
pub async fn list_releases(
    State(state): State<AppState>,
) -> Result<Json<Vec<StudioReleaseDto>>, AppError> {
    // Cuando conectemos auth/roles: filtrar por owner_id del usuario autenticado (o todo si es ADMIN)
    let tracks = Track::find().all(&state.db).await?;

    let releases = tracks
        .into_iter()
        .map(|t| StudioReleaseDto {
            id: t.id,
            title: t.title,
            artist: t.artist,
            media_kind: parse_media_kind(&t.media_kind),
            source_kind: parse_source_kind(&t.source_kind),
            status: PublishStatus::Published,
            duration_secs: t.duration_secs,
            is_ad: t.is_ad,
            images: Vec::new(),
        })
        .collect();

    Ok(Json(releases))
}