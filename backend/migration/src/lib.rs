use sea_orm_migration::prelude::*;

mod m20260101_000001_init;
mod m20261125_000002_create_tracks;
mod m20261125_000003_create_track_assets;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260101_000001_init::Migration),
            Box::new(m20261125_000002_create_tracks::Migration),
            Box::new(m20261125_000003_create_track_assets::Migration),
        ]
    }
}