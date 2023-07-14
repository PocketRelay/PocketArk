use std::collections::HashMap;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::models::{
    auth::Sku,
    character::{CharacterEquipment, CustomizationEntry, SkillTreeEntry, Xp},
};

use chrono::NaiveDateTime;

use super::ValueMap;

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "characters")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    pub character_id: Uuid,
    #[sea_orm(ignore)]
    pub sku: Sku,
    #[serde(skip)]
    pub user_id: u32,
    #[serde(rename = "characterClassName")]
    pub class_name: Uuid,
    pub name: Uuid,
    pub level: u32,
    pub xp: Xp,
    pub promotion: u32,
    pub points: PointMap,
    pub points_spent: PointMap,
    pub points_granted: PointMap,
    pub skill_trees: SkillTree,
    pub attributes: ValueMap,
    pub bonus: ValueMap,
    pub equipments: EquipmentList,
    pub customization: CustomizationMap,
    pub play_stats: ValueMap,
    pub inventory_namespace: String,
    pub last_used: Option<DateTimeUtc>,
    pub promotable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct PointMap(pub HashMap<String, u32>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct CustomizationMap(pub HashMap<String, CustomizationEntry>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct EquipmentList(pub Vec<CharacterEquipment>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SkillTree(pub Vec<SkillTreeEntry>);

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
