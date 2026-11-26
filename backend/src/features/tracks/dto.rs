use serde::{Deserialize, Serialize};

use crate::features::catalog::dto::{
    ImageObject, MediaAssetDto, MediaKind, PublishStatus, SourceKind,
};

#[derive(Debug, Deserialize)]
pub struct CreateTrackDto {
    pub title: String,
    pub artist: String,
    pub media_kind: Option<String>,
    pub source_kind: Option<String>,
    pub status: Option<PublishStatus>,
    pub owner_id: Option<i32>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: Option<i32>,
    pub is_ad: Option<bool>,
    pub images: Option<Vec<ImageObject>>,
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    pub chunk_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub media_kind: MediaKind,
    pub source_kind: SourceKind,
    pub status: PublishStatus,
    pub owner_id: Option<i32>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: i32,
    pub is_ad: bool,
    pub images: Vec<ImageObject>,
    pub asset: Option<MediaAssetDto>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackDto {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub media_kind: Option<MediaKind>,
    pub status: Option<PublishStatus>,
    pub duration_secs: Option<i32>,
    pub is_ad: Option<bool>,
    pub images: Option<Vec<ImageObject>>,
}