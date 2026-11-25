use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTrackDto {
    pub title: String,
    pub artist: String,
    pub media_kind: Option<String>,      // "AUDIO" | "VIDEO"
    pub source_kind: Option<String>,     // "LOCAL" | "EXTERNAL"
    pub external_source: Option<String>, // ej. "deezer", "youtube"
    pub external_id: Option<String>,
    pub duration_secs: Option<i32>,
    pub is_ad: Option<bool>,             // preparado para anuncios en plan FREE
    // Si es LOCAL y tiene archivo en disco:
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    pub chunk_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TrackAssetInfo {
    pub file_size: i64,
    pub chunk_size: i32,
    pub total_chunks: i32,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub media_kind: String,
    pub source_kind: String,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: i32,
    pub is_ad: bool,
    pub asset: Option<TrackAssetInfo>,
}