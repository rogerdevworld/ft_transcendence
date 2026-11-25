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
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub duration_secs: i32,
    pub is_ad: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track_asset::Entity")]
    TrackAssets,
}

impl Related<super::track_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrackAssets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}