use super::{User, ValueMap};
use crate::{
    database::{entity::ClassData, DbResult},
    http::models::{
        auth::Sku,
        character::{CharacterEquipment, Class, CustomizationEntry, SkillTreeEntry, Xp},
    },
    services::defs::{Definitions, LevelTables},
    state::App,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    entity::prelude::*,
    ActiveValue::{self, NotSet, Set},
    IntoActiveModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "challenge_progress")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    #[serde(skip)]
    pub user_id: u32,

    pub challenge_id: Uuid,
    pub counters: ChallengeCounters,
    pub state: String,
    pub times_completed: u32,
    pub last_changed: DateTime<Utc>,
    pub rewarded: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct ChallengeCounters(Vec<ChallengeProgressCounter>);

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeProgressCounter {
    pub name: String,
    pub times_completed: u32,
    pub total_count: u32,
    pub current_count: u32,
    pub target_count: u32,
    pub reset_count: u32,
    pub last_changed: DateTime<Utc>,
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
    pub async fn find_by_user(db: &DatabaseConnection, user: &User) -> DbResult<Vec<Self>> {
        user.find_related(Entity).all(db).await
    }
}
