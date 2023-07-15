use std::collections::HashMap;

use super::User;
use crate::{
    database::DbResult,
    http::models::character::{CharacterEquipment, Xp},
};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{entity::prelude::*, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use uuid::uuid;

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SharedStats(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SharedProgressionList(pub Vec<SharedProgression>);

#[derive(Debug, Clone, Default, Serialize, PartialEq, Deserialize, FromJsonQueryResult)]
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
    pub async fn create_default(user: &User, db: &DatabaseConnection) -> DbResult<Model> {
        // Create models from initial item defs
        let active_character = uuid!("af3a2cf0-dff7-4ca8-9199-73ce546c3e7b"); // HUMAN MALE SOLDIER;

        let mut shared_stats = HashMap::new();
        shared_stats.insert(
            "pathfinderRating".to_string(),
            serde_json::Value::Number(Number::from_f64(0.0).unwrap()),
        );

        let model = ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            active_character_id: Set(active_character),
            shared_equipment: Set(Default::default()),
            shared_progression: Set(Default::default()),
            shared_stats: Set(SharedStats(shared_stats)),
        };

        model.insert(db).await
    }

    pub async fn get_from_user(user: &User, db: &DatabaseConnection) -> DbResult<Model> {
        match user.find_related(Entity).one(db).await? {
            Some(value) => Ok(value),
            None => Self::create_default(user, db).await,
        }
    }

    pub async fn set_shared_equipment(
        user: &User,
        list: Vec<CharacterEquipment>,
        db: &DatabaseConnection,
    ) -> DbResult<Self> {
        let shared_data = Self::get_from_user(user, db).await?;
        let mut shared_data = shared_data.into_active_model();
        shared_data.shared_equipment = Set(CharacterSharedEquipment { list });
        shared_data.update(db).await
    }

    pub async fn set_active_character(
        user: &User,
        uuid: Uuid,
        db: &DatabaseConnection,
    ) -> DbResult<Self> {
        let shared_data = Self::get_from_user(user, db).await?;
        let mut shared_data = shared_data.into_active_model();
        shared_data.active_character_id = Set(uuid);
        shared_data.update(db).await
    }
}
