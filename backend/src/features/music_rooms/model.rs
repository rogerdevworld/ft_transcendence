pub use crate::entities::music_room::{ActiveModel, Column, Entity, Model};

pub fn normalize_visibility(value: Option<String>) -> String {
    match value
        .unwrap_or_else(|| "PUBLIC".to_string())
        .trim()
        .to_uppercase()
        .as_str()
    {
        "PRIVATE" => "PRIVATE".to_string(),
        _ => "PUBLIC".to_string(),
    }
}

pub fn normalize_vote_license(value: Option<String>) -> String {
    match value
        .unwrap_or_else(|| "EVERYONE".to_string())
        .trim()
        .to_uppercase()
        .as_str()
    {
        "HOST_ONLY" => "HOST_ONLY".to_string(),
        "MEMBERS_ONLY" => "MEMBERS_ONLY".to_string(),
        _ => "EVERYONE".to_string(),
    }
}