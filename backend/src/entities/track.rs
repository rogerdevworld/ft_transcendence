use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tracks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub media_kind: String,
    pub source_kind: String,
    pub status: String,
    pub owner_id: Option<i32>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: i32,
    pub is_ad: bool,
    pub images: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track_asset::Entity")]
    TrackAssets,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Owner,
}

impl Related<super::track_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrackAssets.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}