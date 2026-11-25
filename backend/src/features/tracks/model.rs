pub use crate::entities::track::{ActiveModel, Column, Entity, Model};
pub use crate::entities::track_asset::{
    ActiveModel as AssetActiveModel, Entity as TrackAsset, Model as AssetModel,
};

pub const DEFAULT_CHUNK_SIZE: i32 = 262_144; // 256 KB

pub fn normalize_media_kind(value: Option<String>) -> String {
    match value
        .unwrap_or_else(|| "AUDIO".to_string())
        .trim()
        .to_uppercase()
        .as_str()
    {
        "VIDEO" => "VIDEO".to_string(),
        _ => "AUDIO".to_string(),
    }
}

pub fn normalize_source_kind(value: Option<String>, has_external: bool) -> String {
    if let Some(v) = value {
        match v.trim().to_uppercase().as_str() {
            "EXTERNAL" => return "EXTERNAL".to_string(),
            "LOCAL" => return "LOCAL".to_string(),
            _ => {}
        }
    }
    if has_external {
        "EXTERNAL".to_string()
    } else {
        "LOCAL".to_string()
    }
}

pub fn calculate_total_chunks(file_size: i64, chunk_size: i32) -> i32 {
    if file_size <= 0 || chunk_size <= 0 {
        return 0;
    }
    ((file_size + chunk_size as i64 - 1) / chunk_size as i64) as i32
}