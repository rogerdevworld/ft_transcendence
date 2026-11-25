use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::{
    error::AppError,
    state::AppState,
    features::tracks::{
        dto::{CreateTrackDto, TrackAssetInfo, TrackResponse},
        model::{
            self, calculate_total_chunks, normalize_media_kind, normalize_source_kind,
            AssetActiveModel, Entity as Track, TrackAsset, DEFAULT_CHUNK_SIZE,
        },
    },
};

fn to_response(
    track: model::Model,
    asset: Option<crate::entities::track_asset::Model>,
) -> TrackResponse {
    TrackResponse {
        id: track.id,
        title: track.title,
        artist: track.artist,
        media_kind: track.media_kind,
        source_kind: track.source_kind,
        external_source: track.external_source,
        external_id: track.external_id,
        duration_secs: track.duration_secs,
        is_ad: track.is_ad,
        asset: asset.map(|a| TrackAssetInfo {
            file_size: a.file_size,
            chunk_size: a.chunk_size,
            total_chunks: a.total_chunks,
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

    // Si se indica file_path, validamos que exista en disco antes de insertar
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
        external_source: Set(payload.external_source),
        external_id: Set(payload.external_id),
        duration_secs: Set(payload.duration_secs.unwrap_or(0)),
        is_ad: Set(payload.is_ad.unwrap_or(false)),
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