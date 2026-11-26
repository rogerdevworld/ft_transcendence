use serde::{ Serialize, Deserialize };

use crate::features::catalog::dto::UserRole;
use crate::features::users::dto::UserResponse;

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginChallengeResponse {
    pub requires_2fa: bool,
    pub challenge_id: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FaDto {
    pub challenge_id: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserResponse,
}