use std::collections::HashMap;

use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};

use crate::{
    database::DbResult,
    http::models::character::{CharacterEquipment, Xp},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shared_data")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    #[serde(skip)]
    pub user_id: u32,
    #[serde(default)]
    pub active_character_id: Uuid,
    pub shared_stats: SharedStats,
    pub shared_equipment: CharacterSharedEquipment,
    pub shared_progression: SharedProgressionList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SharedStats(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SharedProgressionList(pub Vec<SharedProgression>);

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, FromJsonQueryResult)]
pub struct CharacterSharedEquipment {
    pub list: Vec<CharacterEquipment>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SharedProgression {
    pub name: Uuid,
    pub i18n_name: String,
    pub i18n_description: String,
    pub level: u32,
    pub xp: Xp,
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

impl Model {
    pub async fn set_active_character(
        self,
        character_id: Uuid,
        db: &DatabaseConnection,
    ) -> DbResult<Self> {
        let mut model = self.into_active_model();
        model.active_character_id = ActiveValue::Set(character_id);
        let value = model.update(db).await?;
        Ok(value)
    }
}
