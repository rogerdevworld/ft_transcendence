use serde::Serialize;

use crate::features::catalog::dto::{ImageObject, MediaKind, PublishStatus, SourceKind};

#[derive(Debug, Serialize)]
pub struct StudioReleaseDto {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub media_kind: MediaKind,
    pub source_kind: SourceKind,
    pub status: PublishStatus,
    pub duration_secs: i32,
    pub is_ad: bool,
    pub images: Vec<ImageObject>,
}