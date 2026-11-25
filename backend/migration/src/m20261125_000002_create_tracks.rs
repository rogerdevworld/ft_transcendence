use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tracks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tracks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tracks::Title).string().not_null())
                    .col(ColumnDef::new(Tracks::Artist).string().not_null())
                    // AUDIO | VIDEO
                    .col(
                        ColumnDef::new(Tracks::MediaKind)
                            .string()
                            .not_null()
                            .default("AUDIO"),
                    )
                    // LOCAL | EXTERNAL
                    .col(
                        ColumnDef::new(Tracks::SourceKind)
                            .string()
                            .not_null()
                            .default("LOCAL"),
                    )
                    .col(ColumnDef::new(Tracks::ExternalSource).string().null())
                    .col(ColumnDef::new(Tracks::ExternalId).string().null())
                    .col(
                        ColumnDef::new(Tracks::DurationSecs)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // Gancho futuro para anuncios en planes FREE
                    .col(
                        ColumnDef::new(Tracks::IsAd)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Tracks::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tracks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Tracks {
    Table,
    Id,
    Title,
    Artist,
    MediaKind,
    SourceKind,
    ExternalSource,
    ExternalId,
    DurationSecs,
    IsAd,
    CreatedAt,
}