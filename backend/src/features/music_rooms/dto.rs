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

#[derive(Debug, Clone, Serialize)]
pub struct QueuedTrack {
    pub track_id: i64,
    pub votes: i64,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueTrackRequest {
    pub track_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct VoteTrackRequest {
    pub user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct VoteResponse {
    pub track_id: i64,
    pub voted: bool,
    pub votes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlaying {
    pub track_id: i64,
    pub started_at_ms: i64,
    pub is_playing: bool,
}
