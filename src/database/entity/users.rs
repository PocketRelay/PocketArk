use sea_orm::entity::prelude::*;

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
