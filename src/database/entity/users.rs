use super::{Currency, SharedData, StrikeTeam};
use crate::database::{entity::InventoryItem, DbResult};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{NotSet, Set};

pub type UserId = u32;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub username: String,
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::currency::Entity")]
    Currencies,
    #[sea_orm(has_many = "super::characters::Entity")]
    Characters,
    #[sea_orm(has_many = "super::inventory_items::Entity")]
    InventoryItems,
    #[sea_orm(has_many = "super::class_data::Entity")]
    ClassData,
    #[sea_orm(has_many = "super::seen_articles::Entity")]
    SeenArticles,
    #[sea_orm(has_many = "super::challenge_progress::Entity")]
    ChallengeProgress,
    #[sea_orm(has_one = "super::shared_data::Entity")]
    SharedData,
    #[sea_orm(has_many = "super::strike_teams::Entity")]
    StrikeTeams,
}

impl Model {
    pub async fn create_user(
        username: String,
        password: String,
        db: &DatabaseConnection,
    ) -> DbResult<Self> {
        let user = ActiveModel {
            id: NotSet,
            username: Set(username),
            password: Set(password),
        }
        .insert(db)
        .await?;

        InventoryItem::create_default(db, &user).await?;
        Currency::create_default(db, &user).await?;
        SharedData::create_default(db, &user).await?;
        StrikeTeam::create_default(db, &user).await?;

        Ok(user)
    }

    pub async fn get_user(db: &DatabaseConnection, id: u32) -> DbResult<Option<Self>> {
        Entity::find_by_id(id).one(db).await
    }

    pub async fn get_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> DbResult<Option<Self>> {
        Entity::find()
            .filter(Column::Username.eq(username))
            .one(db)
            .await
    }
}

impl Related<super::currency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Currencies.def()
    }
}

impl Related<super::characters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Characters.def()
    }
}

impl Related<super::inventory_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InventoryItems.def()
    }
}

impl Related<super::class_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClassData.def()
    }
}

impl Related<super::seen_articles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SeenArticles.def()
    }
}

impl Related<super::shared_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SharedData.def()
    }
}

impl Related<super::challenge_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChallengeProgress.def()
    }
}

impl Related<super::strike_teams::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StrikeTeams.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
