use serde::Serialize;

use crate::features::catalog::dto::{ImageObject, MediaAssetDto, MediaKind, VideoChunkDto};

#[derive(Debug, Serialize)]
pub struct ImageUploadResponse {
    pub image: ImageObject,
    pub stored_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
pub struct MediaUploadResponse {
    pub track_id: i32,
    pub title: String,
    pub artist: String,
    pub media_kind: MediaKind,
    pub sha256: String,
    pub asset: MediaAssetDto,
    pub video_chunks: Option<Vec<VideoChunkDto>>,
}