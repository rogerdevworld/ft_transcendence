use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, Set};
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;
use sha2::{Digest, Sha256};

use crate::{
    error::AppError,
    features::{
        catalog::dto::{ImageObject, MediaAssetDto, MediaKind, VideoChunkDto, },
        uploads::dto::{ImageUploadResponse, MediaUploadResponse},
        media_delivery::handlers::build_chunk_metadata,
        tracks::model::{
            self, calculate_total_chunks, AssetActiveModel, DEFAULT_CHUNK_SIZE,
        },
    },
    state::AppState,
};

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_MEDIA_BYTES: usize = 500 * 1024 * 1024; // 500 MB (ajustable por env)

fn uploads_root_dir() -> PathBuf {
    std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "./uploads".to_string()).into()
}

fn public_media_base_url() -> String {
    std::env::var("MEDIA_BASE_URL").unwrap_or_else(|_| "/media".to_string())
}

fn extension_for_image_mime(mime: &str) -> Result<&'static str, AppError> {
    match mime.to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        _ => Err(AppError::BadRequest(format!(
            "Tipo de imagen no soportado: {}",
            mime
        ))),
    }
}

fn extension_and_kind_for_media_mime(
    mime: &str,
    explicit_kind: Option<MediaKind>,
) -> Result<(&'static str, MediaKind), AppError> {
    let lower = mime.to_ascii_lowercase();
    if lower.starts_with("video/") {
        let ext = match lower.as_str() {
            "video/mp4" => "mp4",
            "video/webm" => "webm",
            "video/quicktime" => "mov",
            _ => "mp4",
        };
        Ok((ext, explicit_kind.unwrap_or(MediaKind::Video)))
    } else if lower.starts_with("audio/") {
        let ext = match lower.as_str() {
            "audio/mpeg" | "audio/mp3" => "mp3",
            "audio/ogg" => "ogg",
            "audio/wav" => "wav",
            "audio/flac" => "flac",
            "audio/aac" => "aac",
            _ => "mp3",
        };
        Ok((ext, explicit_kind.unwrap_or(MediaKind::Audio)))
    } else {
        Err(AppError::BadRequest(format!(
            "Tipo MIME de audio/vídeo no soportado: {}",
            mime
        )))
    }
}

fn parse_media_kind_opt(raw: &str) -> Option<MediaKind> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "VIDEO" => Some(MediaKind::Video),
        "PODCAST" => Some(MediaKind::Podcast),
        "AUDIOBOOK" => Some(MediaKind::Audiobook),
        "AUDIO" => Some(MediaKind::Audio),
        _ => None,
    }
}

fn compute_sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn build_video_chunks_with_hash(
    data: &[u8],
    chunk_size: u64,
    total_chunks: u64,
) -> Vec<VideoChunkDto> {
    let total_len = data.len() as u64;
    (0..total_chunks)
        .map(|index| {
            let offset = index * chunk_size;
            let remaining = total_len.saturating_sub(offset);
            let size = remaining.min(chunk_size);
            let start = offset as usize;
            let end = (offset + size) as usize;
            let chunk_hash = compute_sha256_hex(&data[start..end]);

            VideoChunkDto {
                index,
                offset,
                size,
                hash: Some(chunk_hash),
            }
        })
        .collect()
}

// POST /api/uploads/images
pub async fn upload_image(
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ImageUploadResponse>), AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut mime_type: Option<String> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("El campo multipart 'file' es obligatorio".to_string())
    })?;
    let sha256 = compute_sha256_hex(&bytes);
    let mime = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let ext = extension_for_image_mime(&mime)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let detected_mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Error leyendo archivo: {}", e)))?;

                if data.is_empty() || data.len() > MAX_IMAGE_BYTES {
                    return Err(AppError::BadRequest(
                        "Tamaño de imagen inválido o superior al límite permitido".to_string(),
                    ));
                }

                mime_type = Some(detected_mime);
                file_bytes = Some(data.to_vec());
            }
            "width" => {
                let txt = field.text().await.unwrap_or_default();
                width = txt.trim().parse::<u32>().ok();
            }
            "height" => {
                let txt = field.text().await.unwrap_or_default();
                height = txt.trim().parse::<u32>().ok();
            }
            _ => {}
        }
    }

    let dir = uploads_root_dir().join("images");
    fs::create_dir_all(&dir)
        .await
        .map_err(|_| AppError::BadRequest("No se pudo crear directorio de imágenes".to_string()))?;

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let full_path = dir.join(&filename);
    let mut file = fs::File::create(&full_path)
        .await
        .map_err(|_| AppError::BadRequest("No se pudo crear el archivo de imagen".to_string()))?;
    file.write_all(&bytes)
        .await
        .map_err(|_| AppError::BadRequest("Error guardando la imagen en disco".to_string()))?;

    let size_bytes = bytes.len() as u64;
    let stored_path = full_path.to_string_lossy().to_string();
    let url = format!(
        "{}/images/{}",
        public_media_base_url().trim_end_matches('/'),
        filename
    );

    Ok((
        StatusCode::CREATED,
        Json(ImageUploadResponse {
            image: ImageObject {
                url,
                width,
                height,
            },
            stored_path,
            size_bytes,
            sha256,
        }),
    ))
}

// POST /api/uploads/media
pub async fn upload_media(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaUploadResponse>), AppError> {
    let mut title: Option<String> = None;
    let mut artist: Option<String> = None;
    let mut requested_kind: Option<MediaKind> = None;
    let mut duration_secs: Option<i32> = None;
    let mut chunk_size_opt: Option<i32> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Error leyendo multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => {
                let v = field.text().await.unwrap_or_default().trim().to_string();
                if !v.is_empty() {
                    title = Some(v);
                }
            }
            "artist" => {
                let v = field.text().await.unwrap_or_default().trim().to_string();
                if !v.is_empty() {
                    artist = Some(v);
                }
            }
            "media_kind" => {
                let v = field.text().await.unwrap_or_default();
                requested_kind = parse_media_kind_opt(&v);
            }
            "duration_secs" => {
                let v = field.text().await.unwrap_or_default();
                duration_secs = v.trim().parse::<i32>().ok();
            }
            "chunk_size" => {
                let v = field.text().await.unwrap_or_default();
                chunk_size_opt = v.trim().parse::<i32>().ok();
            }
            "file" => {
                let detected_mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Error leyendo archivo: {}", e)))?;

                if data.is_empty() || data.len() > MAX_MEDIA_BYTES {
                    return Err(AppError::BadRequest(
                        "Tamaño de archivo inválido o superior al límite permitido".to_string(),
                    ));
                }

                mime_type = Some(detected_mime);
                file_bytes = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let title = title
        .ok_or_else(|| AppError::BadRequest("El campo 'title' es obligatorio".to_string()))?;
    let artist = artist
        .ok_or_else(|| AppError::BadRequest("El campo 'artist' es obligatorio".to_string()))?;
    let bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("El campo multipart 'file' es obligatorio".to_string())
    })?;
    let sha256 = compute_sha256_hex(&bytes);
    let mime = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let (ext, media_kind) = extension_and_kind_for_media_mime(&mime, requested_kind)?;

    let subdir = match media_kind {
        MediaKind::Video => "videos",
        _ => "audio",
    };
    let dir = uploads_root_dir().join(subdir);
    fs::create_dir_all(&dir)
        .await
        .map_err(|_| AppError::BadRequest("No se pudo crear directorio de medios".to_string()))?;

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let full_path = dir.join(&filename);
    let mut file = fs::File::create(&full_path)
        .await
        .map_err(|_| AppError::BadRequest("No se pudo crear el archivo en disco".to_string()))?;
    file.write_all(&bytes)
        .await
        .map_err(|_| AppError::BadRequest("Error escribiendo el archivo en disco".to_string()))?;

    let file_size_u64 = bytes.len() as u64;
    let file_size_i64 = file_size_u64 as i64;
    let chunk_size_i32 = chunk_size_opt.unwrap_or(DEFAULT_CHUNK_SIZE).max(1024);
    let total_chunks_i32 = calculate_total_chunks(file_size_i64, chunk_size_i32);

    let media_kind_str = match media_kind {
        MediaKind::Audio => "AUDIO",
        MediaKind::Video => "VIDEO",
        MediaKind::Podcast => "PODCAST",
        MediaKind::Audiobook => "AUDIOBOOK",
    };

    let created_track = model::ActiveModel {
        title: Set(title.clone()),
        artist: Set(artist.clone()),
        media_kind: Set(media_kind_str.to_string()),
        source_kind: Set("LOCAL".to_string()),
        external_source: Set(None),
        external_id: Set(None),
        duration_secs: Set(duration_secs.unwrap_or(0)),
        is_ad: Set(false),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let stored_path = full_path.to_string_lossy().to_string();
    let _created_asset = AssetActiveModel {
        track_id: Set(created_track.id),
        file_path: Set(stored_path),
        mime_type: Set(mime.clone()),
        file_size: Set(file_size_i64),
        chunk_size: Set(chunk_size_i32),
        total_chunks: Set(total_chunks_i32),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let chunk_size_u64 = chunk_size_i32 as u64;
    let total_chunks_u64 = total_chunks_i32 as u64;

    let video_chunks = if matches!(media_kind, MediaKind::Video) {
        Some(build_video_chunks_with_hash(
            &bytes,
            chunk_size_u64,
            total_chunks_u64,
        ))
    } else {
        None
    };

    Ok((
        StatusCode::CREATED,
        Json(MediaUploadResponse {
            track_id: created_track.id,
            title,
            artist,
            media_kind,
            sha256,
            asset: MediaAssetDto {
                file_size: file_size_u64,
                chunk_size: chunk_size_u64,
                total_chunks: total_chunks_u64,
                mime_type: mime,
            },
            video_chunks,
        }),
    ))
}