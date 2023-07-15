use sea_orm::entity::prelude::*;

use crate::database::DbResult;

use super::{Currency, SharedData};

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
    pub async fn get_shared_data(&self, db: &DatabaseConnection) -> DbResult<SharedData> {
        let data = self
            .find_related(super::shared_data::Entity)
            .one(db)
            .await?;
        if let Some(data) = data {
            return Ok(data);
        }

        let new_data = super::shared_data::ActiveModel {
            ..Default::default()
        };
        new_data.insert(db).await
    }

    pub async fn get_currencies(&self, db: &DatabaseConnection) -> DbResult<Vec<Currency>> {
        self.find_related(super::currency::Entity).all(db).await
    }
}
