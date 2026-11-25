use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "track_assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub track_id: i32,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub chunk_size: i32,
    pub total_chunks: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::track::Entity",
        from = "Column::TrackId",
        to = "super::track::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Track,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Track.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}