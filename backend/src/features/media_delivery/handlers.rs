use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::io::SeekFrom;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    entities::track_asset::{Column as AssetColumn, Entity as TrackAsset},
    error::AppError,
    features::catalog::dto::VideoChunkDto,
    state::AppState,
};

async fn read_file_slice(
    file_path: &str,
    start: u64,
    length: usize,
) -> Result<Vec<u8>, AppError> {
    let mut file = File::open(file_path)
        .await
        .map_err(|_| AppError::NotFound("Archivo no encontrado en disco".to_string()))?;

    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| AppError::BadRequest("Error haciendo seek en el archivo".to_string()))?;

    let mut buffer = vec![0u8; length];
    file.read_exact(&mut buffer)
        .await
        .map_err(|_| AppError::BadRequest("Error leyendo bytes del archivo".to_string()))?;

    Ok(buffer)
}

pub fn build_chunk_metadata(
    file_size: u64,
    chunk_size: u64,
    total_chunks: u64,
) -> Vec<VideoChunkDto> {
    (0..total_chunks)
        .map(|index| {
            let offset = index * chunk_size;
            let remaining = file_size.saturating_sub(offset);
            let size = remaining.min(chunk_size);
            VideoChunkDto {
                index,
                offset,
                size,
                hash: None,
            }
        })
        .collect()
}

// GET /api/tracks/{id}/chunks/{index}
pub async fn get_chunk(
    State(state): State<AppState>,
    Path((id, index)): Path<(i32, i32)>,
) -> Result<Response, AppError> {
    let asset = TrackAsset::find()
        .filter(AssetColumn::TrackId.eq(id))
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "No hay asset local con chunks para la pista {}",
                id
            ))
        })?;

    if index < 0 || index >= asset.total_chunks {
        return Err(AppError::BadRequest(format!(
            "Índice de chunk inválido (rango válido: 0..{})",
            asset.total_chunks.saturating_sub(1)
        )));
    }

    let offset = (index as u64) * (asset.chunk_size as u64);
    let remaining = (asset.file_size as u64).saturating_sub(offset);
    let to_read = remaining.min(asset.chunk_size as u64) as usize;

    let buffer = read_file_slice(&asset.file_path, offset, to_read).await?;

    let mut res = Response::new(Body::from(buffer));
    *res.status_mut() = StatusCode::OK;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&asset.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    res.headers_mut().insert(
        header::ACCEPT_RANGES,
        HeaderValue::from_static("bytes"),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("x-chunk-index"),
        HeaderValue::from_str(&index.to_string()).unwrap(),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("x-total-chunks"),
        HeaderValue::from_str(&asset.total_chunks.to_string()).unwrap(),
    );

    Ok(res)
}

// GET /api/tracks/{id}/stream (HTTP Range Requests -> 206 Partial Content)
pub async fn stream_range(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let asset = TrackAsset::find()
        .filter(AssetColumn::TrackId.eq(id))
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "No hay asset local para streaming de la pista {}",
                id
            ))
        })?;

    let total_size = asset.file_size as u64;
    if total_size == 0 {
        return Err(AppError::BadRequest("El archivo está vacío".to_string()));
    }

    let (start, end) = if let Some(range_header) = headers.get(header::RANGE) {
        let range_str = range_header.to_str().unwrap_or("");
        let bytes_part = range_str
            .strip_prefix("bytes=")
            .ok_or_else(|| AppError::BadRequest("Cabecera Range inválida".to_string()))?;

        if let Some(suffix) = bytes_part.strip_prefix('-') {
            let suffix_len: u64 = suffix
                .parse()
                .map_err(|_| AppError::BadRequest("Rango de sufijo inválido".to_string()))?;
            if suffix_len == 0 {
                return Ok(StatusCode::RANGE_NOT_SATISFIABLE.into_response());
            }
            let start = total_size.saturating_sub(suffix_len);
            (start, total_size - 1)
        } else {
            let mut parts = bytes_part.split('-');
            let start: u64 = parts
                .next()
                .and_then(|s| if s.is_empty() { None } else { s.parse().ok() })
                .ok_or_else(|| AppError::BadRequest("Rango inicial inválido".to_string()))?;
            let end: u64 = parts
                .next()
                .and_then(|s| if s.is_empty() { None } else { s.parse().ok() })
                .unwrap_or_else(|| (start + 1_048_575).min(total_size - 1));

            if start >= total_size || end >= total_size || start > end {
                return Ok(StatusCode::RANGE_NOT_SATISFIABLE.into_response());
            }
            (start, end)
        }
    } else {
        (0, (1_048_575).min(total_size - 1))
    };

    let length = (end - start + 1) as usize;
    let buffer = read_file_slice(&asset.file_path, start, length).await?;

    let content_range = format!("bytes {}-{}/{}", start, end, total_size);
    let mut res = Response::new(Body::from(buffer));
    *res.status_mut() = StatusCode::PARTIAL_CONTENT;

    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&asset.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    res.headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    res.headers_mut().insert(
        header::CONTENT_RANGE,
        HeaderValue::from_str(&content_range).unwrap(),
    );

    Ok(res)
}