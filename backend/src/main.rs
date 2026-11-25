// backend/src/main.rs
mod config;
mod entities;
mod error;
mod routes;
mod state;
mod features;
// mod health;
// mod users;
// mod music_rooms;
// mod media_delivery;
// mod tracks;
// mod live_streams;

use axum::Router;
use migration::Migrator;
use sea_orm_migration::prelude::MigratorTrait;
use sea_orm::Database;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::Config, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            tracing::error!("Error cargando configuración: {}", err);
            std::process::exit(1);
        }
    };

    // Conectar pool de SeaORM a mariadb
    let db = match Database::connect(&config.database_url).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Error conectando a mariadb: {}", err);
            std::process::exit(1);
        }
    };

    // Aplicar migraciones pendientes al arrancar
    if let Err(err) = Migrator::up(&db, None).await {
        tracing::error!("Error aplicando migraciones: {}", err);
        std::process::exit(1);
    }

    let addr = config.bind_addr();
    // let config = Config::from_env().expect("Error cargando configuración");

    let db = Database::connect(&config.database_url)
        .await
        .expect("Error conectando a MariaDB");

    Migrator::up(&db, None)
        .await
        .expect("Error ejecutando migraciones");

    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("URL de Redis inválida");

    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Error conectando a Redis");

    let ws_hub = crate::features::music_rooms::RoomHub::new();

    let state = AppState::new(config, db, redis_conn, ws_hub);

    let api_routes = Router::new()
        .merge(routes::health::router())
        .merge(crate::features::users::routes::routes())
        .merge(crate::features::music_rooms::routes::routes())
        .merge(crate::features::media_delivery::routes::routes())
        .merge(crate::features::tracks::routes::routes());

    let app = Router::new()
        .nest("/api", api_routes)
        .with_state(state);

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(err) => {
            tracing::error!("No se pudo enlazar a {}: {}", addr, err);
            std::process::exit(1);
        }
    };

    tracing::info!("Servidor Music Room escuchando en http://{}", addr);

    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!("Fallo crítico en el servidor: {}", err);
        std::process::exit(1);
    }
}