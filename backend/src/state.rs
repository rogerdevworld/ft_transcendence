use crate::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
    pub ws_hub: RoomHub,
}

use crate::features::music_rooms::RoomHub;

impl AppState {
    pub fn new(config: Config, db: DatabaseConnection, redis: ConnectionManager, ws_hub: RoomHub) -> Self {
        Self {
            config: Arc::new(config),
            db,
            redis,
            ws_hub,
        }
    }
}