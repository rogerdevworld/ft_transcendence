use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Tabla users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).string())
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(
                        ColumnDef::new(Users::SubscriptionPlan)
                            .string()
                            .not_null()
                            .default("FREE"),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Tabla music_rooms (con FK a users.id)
        manager
            .create_table(
                Table::create()
                    .table(MusicRooms::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MusicRooms::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MusicRooms::OwnerId).integer().not_null())
                    .col(ColumnDef::new(MusicRooms::Title).string().not_null())
                    .col(
                        ColumnDef::new(MusicRooms::Visibility)
                            .string()
                            .not_null()
                            .default("PUBLIC"),
                    )
                    .col(
                        ColumnDef::new(MusicRooms::VoteLicense)
                            .string()
                            .not_null()
                            .default("EVERYONE"),
                    )
                    .col(
                        ColumnDef::new(MusicRooms::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_music_rooms_owner")
                            .from(MusicRooms::Table, MusicRooms::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MusicRooms::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    DisplayName,
    SubscriptionPlan,
    CreatedAt,
}

#[derive(Iden)]
enum MusicRooms {
    Table,
    Id,
    OwnerId,
    Title,
    Visibility,
    VoteLicense,
    CreatedAt,
}