use crate::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(config: Config, db: DatabaseConnection) -> Self {
        Self {
            config: Arc::new(config),
            db,
        }
    }
}