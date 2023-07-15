use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{NotSet, Set};

use crate::database::DbResult;

use super::User;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "class_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub user_id: u32,
    pub name: Uuid,
    pub unlocked: bool,
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
    pub async fn create(
        user: &User,
        name: Uuid,
        unlocked: bool,
        db: &DatabaseConnection,
    ) -> DbResult<()> {
        let model = ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            name: Set(name),
            unlocked: Set(unlocked),
        };
        let _ = model.insert(db).await?;
        Ok(())
    }
}
