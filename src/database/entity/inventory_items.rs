use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::ValueMap;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "inventory_items")]
#[serde(rename_all = "camelCase")]

pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    pub item_id: Uuid,
    #[serde(skip)]
    pub user_id: u32,
    pub definition_name: String,
    pub stack_size: u32,
    pub seen: bool,
    pub instance_attributes: ValueMap,
    pub created: DateTimeUtc,
    pub last_grant: DateTimeUtc,
    #[serde(rename = "earndBy")]
    pub earned_by: String,
    pub restricted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
