use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateRoomDto {
    pub owner_id: i32,
    pub title: String,
    pub visibility: Option<String>,
    pub vote_license: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoomDto {
    pub title: Option<String>,
    pub visibility: Option<String>,
    pub vote_license: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: i32,
    pub owner_id: i32,
    pub title: String,
    pub visibility: String,
    pub vote_license: String,
}