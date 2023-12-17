//! Class data database models

use super::users::UserId;
use super::User;
use crate::database::DbResult;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::InsertResult;
use std::future::Future;

/// Class data database structure
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "class_data")]
pub struct Model {
    /// The user this data is for
    #[sea_orm(primary_key)]
    pub user_id: UserId,
    /// The class definition name this data represents
    #[sea_orm(primary_key)]
    pub class_name: Uuid,
    // Whether this class is unlocked
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

impl Model {
    /// Sets the `unlocked` state of the class `class_name` for the
    /// specific `user`. Will create the class data if the user doesn't
    /// already have one.
    pub fn set<'db, C>(
        db: &'db C,
        user: &User,
        class_name: Uuid,
        unlocked: bool,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::insert(ActiveModel {
            user_id: Set(user.id),
            class_name: Set(class_name),
            unlocked: Set(unlocked),
        })
        .on_conflict(
            // Update the value column if a key already exists
            OnConflict::columns([Column::UserId, Column::ClassName])
                // Update the unlocked value
                .update_column(Column::Unlocked)
                .to_owned(),
        )
        .exec(db)
    }

    /// Retrieves all the class data associated with the
    /// specified `user`
    pub fn all<'db, C>(db: &'db C, user: &User) -> impl Future<Output = DbResult<Vec<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db)
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
