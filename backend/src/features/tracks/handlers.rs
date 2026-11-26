use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, QueryFilter, Set};

use crate::{
    error::AppError,
    features::{
        catalog::dto::{
            parse_images, serialize_images, MediaAssetDto, MediaKind, PublishStatus, SourceKind,
        },
        tracks::{
            dto::{CreateTrackDto, TrackResponse, UpdateTrackDto},
            model::{
                self, calculate_total_chunks, normalize_media_kind, normalize_source_kind,
                AssetActiveModel, Entity as Track, TrackAsset, DEFAULT_CHUNK_SIZE,
            },
        },
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

fn status_to_db(status: &PublishStatus) -> String {
    match status {
        PublishStatus::Published => "PUBLISHED".to_string(),
        other => format!("{other:?}").to_ascii_uppercase(),
    }
}

fn parse_publish_status(raw: &str) -> PublishStatus {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DRAFT" => PublishStatus::Draft,
        "ARCHIVED" => PublishStatus::Archived,
        "HIDDEN" => PublishStatus::Hidden,
        _ => PublishStatus::Published,
    }
}

fn to_response(
    track: model::Model,
    asset: Option<crate::entities::track_asset::Model>,
) -> TrackResponse {
    let status = parse_publish_status(&track.status);
    let images = parse_images(&track.images);

    TrackResponse {
        id: track.id,
        title: track.title,
        artist: track.artist,
        media_kind: parse_media_kind(&track.media_kind),
        source_kind: parse_source_kind(&track.source_kind),
        status,
        owner_id: track.owner_id,
        external_source: track.external_source,
        external_id: track.external_id,
        duration_secs: track.duration_secs,
        is_ad: track.is_ad,
        images,
        asset: asset.map(|a| MediaAssetDto {
            file_size: a.file_size.max(0) as u64,
            chunk_size: a.chunk_size.max(0) as u64,
            total_chunks: a.total_chunks.max(0) as u64,
            mime_type: a.mime_type,
        }),
    }
}

// POST /api/tracks
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateTrackDto>,
) -> Result<(StatusCode, Json<TrackResponse>), AppError> {
    let title = payload.title.trim().to_string();
    let artist = payload.artist.trim().to_string();

    if title.is_empty() || artist.is_empty() {
        return Err(AppError::BadRequest(
            "title y artist son obligatorios".to_string(),
        ));
    }

    let media_kind = normalize_media_kind(payload.media_kind);
    let has_external = payload
        .external_id
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let source_kind = normalize_source_kind(payload.source_kind, has_external);
    let publish_status = payload.status.unwrap_or(PublishStatus::Published);
    let images_json = serialize_images(&payload.images);

    let local_file_meta = match &payload.file_path {
        Some(path) if !path.trim().is_empty() => {
            let meta = tokio::fs::metadata(path.trim()).await.map_err(|_| {
                AppError::BadRequest(format!("No se encontró el archivo en {}", path))
            })?;
            Some((path.trim().to_string(), meta.len() as i64))
        }
        _ => None,
    };

    let new_track = model::ActiveModel {
        title: Set(title),
        artist: Set(artist),
        media_kind: Set(media_kind.clone()),
        source_kind: Set(source_kind),
        status: Set(status_to_db(&publish_status)),
        owner_id: Set(payload.owner_id),
        external_source: Set(payload.external_source),
        external_id: Set(payload.external_id),
        duration_secs: Set(payload.duration_secs.unwrap_or(0).max(0)),
        is_ad: Set(payload.is_ad.unwrap_or(false)),
        images: Set(images_json),
        ..Default::default()
    };

    let created_track = new_track.insert(&state.db).await?;

    let created_asset = if let Some((file_path, file_size)) = local_file_meta {
        let chunk_size = payload.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE).max(1024);
        let total_chunks = calculate_total_chunks(file_size, chunk_size);
        let mime_type = payload.mime_type.unwrap_or_else(|| {
            if media_kind == "VIDEO" {
                "video/mp4".to_string()
            } else {
                "audio/mpeg".to_string()
            }
        });

        let new_asset = AssetActiveModel {
            track_id: Set(created_track.id),
            file_path: Set(file_path),
            mime_type: Set(mime_type),
            file_size: Set(file_size),
            chunk_size: Set(chunk_size),
            total_chunks: Set(total_chunks),
            ..Default::default()
        };

        Some(new_asset.insert(&state.db).await?)
    } else {
        None
    };

    Ok((
        StatusCode::CREATED,
        Json(to_response(created_track, created_asset)),
    ))
}

// GET /api/tracks
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TrackResponse>>, AppError> {
    let tracks = Track::find().all(&state.db).await?;
    let mut result = Vec::with_capacity(tracks.len());

    for t in tracks {
        let asset = TrackAsset::find()
            .filter(crate::entities::track_asset::Column::TrackId.eq(t.id))
            .one(&state.db)
            .await?;
        result.push(to_response(t, asset));
    }

    Ok(Json(result))
}

// GET /api/tracks/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<TrackResponse>, AppError> {
    let track = Track::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pista {} no encontrada", id)))?;

    let asset = TrackAsset::find()
        .filter(crate::entities::track_asset::Column::TrackId.eq(track.id))
        .one(&state.db)
        .await?;

    Ok(Json(to_response(track, asset)))
}

// PATCH /api/tracks/{id}
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTrackDto>,
) -> Result<Json<TrackResponse>, AppError> {
    let track = Track::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pista {} no encontrada", id)))?;

    let mut active: model::ActiveModel = track.into();

    if let Some(title) = payload.title {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("title no puede estar vacío".to_string()));
        }
        active.title = Set(trimmed.to_string());
    }

    if let Some(artist) = payload.artist {
        let trimmed = artist.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("artist no puede estar vacío".to_string()));
        }
        active.artist = Set(trimmed.to_string());
    }

    if let Some(kind) = payload.media_kind {
        let raw_kind = match kind {
            MediaKind::Audio => "AUDIO",
            MediaKind::Video => "VIDEO",
            MediaKind::Podcast => "PODCAST",
            MediaKind::Audiobook => "AUDIOBOOK",
        };
        active.media_kind = Set(normalize_media_kind(Some(raw_kind.to_string())));
    }

    if let Some(status) = payload.status {
        active.status = Set(status_to_db(&status));
    }

    if let Some(duration_secs) = payload.duration_secs {
        active.duration_secs = Set(duration_secs.max(0));
    }

    if let Some(is_ad) = payload.is_ad {
        active.is_ad = Set(is_ad);
    }

    if let Some(images) = payload.images {
        active.images = Set(serialize_images(&Some(images)));
    }

    let updated_track = active.update(&state.db).await?;

    let asset = TrackAsset::find()
        .filter(crate::entities::track_asset::Column::TrackId.eq(updated_track.id))
        .one(&state.db)
        .await?;

    Ok(Json(to_response(updated_track, asset)))
}

// DELETE /api/tracks/{id}
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let track = Track::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pista {} no encontrada", id)))?;

    if let Some(asset) = TrackAsset::find()
        .filter(crate::entities::track_asset::Column::TrackId.eq(track.id))
        .one(&state.db)
        .await?
    {
        let _ = tokio::fs::remove_file(&asset.file_path).await;
        let asset_active: crate::entities::track_asset::ActiveModel = asset.into();
        asset_active.delete(&state.db).await?;
    }

    let active: model::ActiveModel = track.into();
    let _res: DeleteResult = active.delete(&state.db).await?;

    Ok(StatusCode::NO_CONTENT)
}