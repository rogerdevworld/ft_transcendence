// backend/src/main.rs
mod config;
mod entities;
mod error;
mod routes;
mod state;

use axum::Router;
use migration::{Migrator, MigratorTrait};
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
    let state = AppState::new(config, db);

    let app = Router::new()
        .nest("/api", routes::health::router())
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