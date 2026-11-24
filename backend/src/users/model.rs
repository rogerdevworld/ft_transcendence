pub use crate::entities::user::{ActiveModel, Column, Entity, Model};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    User,
    Admin,
}

impl UserRole {
    pub fn from_plan_or_default(value: &str) -> Self {
        match value.trim().to_uppercase().as_str() {
            "ADMIN" => Self::Admin,
            _ => Self::User,
        }
    }

    pub fn can_manage_admin_panel(&self) -> bool {
        matches!(self, Self::Admin)
    }
}