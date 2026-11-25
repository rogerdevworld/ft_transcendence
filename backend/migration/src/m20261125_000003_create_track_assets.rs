use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrackAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TrackAssets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TrackAssets::TrackId).integer().not_null())
                    .col(ColumnDef::new(TrackAssets::FilePath).string().not_null())
                    .col(
                        ColumnDef::new(TrackAssets::MimeType)
                            .string()
                            .not_null()
                            .default("audio/mpeg"),
                    )
                    .col(
                        ColumnDef::new(TrackAssets::FileSize)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TrackAssets::ChunkSize)
                            .integer()
                            .not_null()
                            .default(262144),
                    )
                    .col(
                        ColumnDef::new(TrackAssets::TotalChunks)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_track_assets_track")
                            .from(TrackAssets::Table, TrackAssets::TrackId)
                            .to(Tracks::Table, Tracks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrackAssets::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TrackAssets {
    Table,
    Id,
    TrackId,
    FilePath,
    MimeType,
    FileSize,
    ChunkSize,
    TotalChunks,
}

#[derive(Iden)]
enum Tracks {
    Table,
    Id,
}