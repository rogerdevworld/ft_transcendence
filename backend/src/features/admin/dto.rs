use serde::{Deserialize, Serialize};

use crate::features::catalog::dto::UserRole;

#[derive(Debug, Serialize)]
pub struct AdminStatsResponse {
    pub total_users: u64,
    pub total_tracks: u64,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateTrackDto {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration_secs: Option<i32>,
    pub is_ad: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserDto {
    pub display_name: Option<String>,
    pub role: Option<UserRole>,
    pub subscription_plan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminActionResponse {
    pub ok: bool,
    pub message: String,
}