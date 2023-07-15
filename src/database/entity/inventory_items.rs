use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::database::DbResult;

use super::{User, ValueMap};

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

impl Model {
    pub async fn create_default(user: &User, db: &DatabaseConnection) -> DbResult<()> {
        let now = Utc::now();
        // Create models from initial item defs
        let items = [
            "79f3511c-55da-67f0-5002-359c370015d8", // HUMAN FEMALE SOLDIER
            "a3960123-3625-4126-82e4-1f9a127d33aa", // HUMAN MALE ENGINEER
            "baae0381-8690-4097-ae6d-0c16473519b4", // HUMAN MALE SENTINEL
            "c756c741-1bc8-47a8-9f35-b7ca943ba034", // HUMAN FEMALE ENGINEER
            "e4357633-93bc-4596-99c3-4cc0a49b2277", // HUMAN MALE ADEPT
            "7fd30824-e20c-473e-b906-f4f30ebc4bb0", // HUMAN MALE VANGUARD
            "96fa16c5-9f2b-46f8-a491-a4b0a24a1089", // HUMAN FEMALE VANGUARD
            "34aeef66-a030-445e-98e2-1513c0c78df4", // HUMAN MALE INFILTRATOR
            "af3a2cf0-dff7-4ca8-9199-73ce546c3e7b", // HUMAN MALE SOLDIER
            "319ffe5d-f8fb-4217-bd2f-2e8af4f53fc8", // HUMAN FEMALE SENTINEL
            "e2f76cf1-4b42-4dba-9751-f2add5c3f654", // HUMAN FEMALE ADEPT
            "4ccc7f54-791c-4b66-954b-a0bd6496f210", // M-3 PREDATOR
            "d5bf2213-d2d2-f892-7310-c39a15fb2ef3", // M-8 AVENGER
            "38e07595-764b-4d9c-b466-f26c7c416860", // VIPER
            "ca7d0f24-fc19-4a78-9d25-9c84eb01e3a5", // M-23 KATANA
        ]
        .into_iter()
        .map(|definition_name| ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(user.id),
            item_id: ActiveValue::Set(Uuid::new_v4()),
            definition_name: ActiveValue::Set(definition_name.to_string()),
            stack_size: ActiveValue::Set(1),
            seen: ActiveValue::Set(false),
            instance_attributes: ActiveValue::Set(ValueMap::default()),
            created: ActiveValue::Set(now),
            last_grant: ActiveValue::Set(now),
            earned_by: ActiveValue::Set("granted".to_string()),
            restricted: ActiveValue::Set(false),
        });
        Entity::insert_many(items)
            .exec_without_returning(db)
            .await?;
        Ok(())
    }
}
