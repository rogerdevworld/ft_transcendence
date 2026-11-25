use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub password: Option<String>,
    pub display_name: String,
    pub subscription_plan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<String>,
    pub subscription_plan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub subscription_plan: String,
}