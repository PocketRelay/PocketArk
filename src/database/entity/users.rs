use super::{Currency, SharedData};
use crate::database::{
    entity::{Character, InventoryItem},
    DbResult,
};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{NotSet, Set};

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
    #[sea_orm(has_one = "super::shared_data::Entity")]
    SharedData,
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

impl ActiveModelBehavior for ActiveModel {}

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

        InventoryItem::create_default(&user, db).await?;
        Character::create_default(&user, db).await?;
        Currency::create_default(&user, db).await?;
        SharedData::create_default(&user, db).await?;

        Ok(user)
    }

    pub async fn get_user(id: u32, db: &DatabaseConnection) -> DbResult<Option<Self>> {
        Entity::find_by_id(id).one(db).await
    }
}
