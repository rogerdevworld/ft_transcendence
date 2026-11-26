use serde::{Deserialize, Serialize};

use crate::features::catalog::dto::{ImageObject, UserRole};

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub role: Option<UserRole>,
    pub subscription_plan: Option<String>,
    pub country: Option<String>,
    pub images: Option<Vec<ImageObject>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<String>,
    pub subscription_plan: Option<String>,
    pub country: Option<String>,
    pub images: Option<Vec<ImageObject>>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
    pub subscription_plan: String,
    pub country: Option<String>,
    pub images: Vec<ImageObject>,
    pub followers_total: u64,
    pub uri: String,
}