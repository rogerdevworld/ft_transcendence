use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Listener,
    Artist,
    Admin,
    Creator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaKind {
    Audio,
    Video,
    Podcast,
    Audiobook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceKind {
    Local,
    External,
    P2p,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishStatus {
    Draft,
    Processing,
    Published,
    Hidden,
    Rejected,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAssetDto {
    pub file_size: u64,
    pub chunk_size: u64,
    pub total_chunks: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItemDto {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub media_kind: MediaKind,
    pub source_kind: SourceKind,
    pub status: PublishStatus,
    pub owner_id: Option<i64>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: u64,
    pub is_ad: bool,
    pub images: Vec<ImageObject>,
    pub asset: Option<MediaAssetDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedDto<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDto {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoChunkDto {
    pub index: u64,
    pub offset: u64,
    pub size: u64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAssetDto {
    pub file_size: u64,
    pub chunk_size: u64,
    pub total_chunks: u64,
    pub mime_type: String,
    pub chunks: Option<Vec<VideoChunkDto>>,
}

pub fn parse_images(raw: &Option<String>) -> Vec<ImageObject> {
    raw.as_deref()
        .and_then(|s| serde_json::from_str::<Vec<ImageObject>>(s).ok())
        .unwrap_or_default()
}

pub fn serialize_images(images: &Option<Vec<ImageObject>>) -> Option<String> {
    images
        .as_ref()
        .filter(|list| !list.is_empty())
        .and_then(|list| serde_json::to_string(list).ok())
}