//! Inventory item database models
//!
//!
//! Note: when manually querying the database for the `definition_name` column, its stored as
//! a BLOB TEXT, so to query it you must prefix the string with "x" like the following query:
//! ```sql
//! SELECT `definition_name` FROM `inventory_items` WHERE `definition_name` = x'af3a2cf0dff74ca8919973ce546c3e7b'`
//! ```
//! (Don't include hyphens in the definition name)

use super::{users::UserId, SeaGenericMap};
use crate::{
    database::{
        entity::{InventoryItem, User},
        DbResult,
    },
    definitions::items::ItemName,
};
use chrono::Utc;
use futures::Future;
use sea_orm::{
    entity::prelude::*,
    sea_query::{Expr, OnConflict},
    ActiveValue::{NotSet, Set},
    IntoActiveModel, UpdateResult,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Item ID keying has been replaced with integer keys rather than the UUIDs
/// used by the official game, this is because its *very* annoying to work with
/// UUIDs as primary keys in the SQLite database (Basically defeats the purpose of SeaORM)
pub type ItemId = u32;

/// Inventory Item database structure
#[serde_as]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "inventory_items")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[serde(rename = "itemId")]
    #[sea_orm(primary_key)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub id: ItemId,
    #[serde(skip)]
    pub user_id: UserId,
    pub definition_name: ItemName,
    pub stack_size: u32,
    pub seen: bool,
    pub instance_attributes: SeaGenericMap,
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

impl Model {
    /// Adds an item for the provided player. If an item with a matching `definition_name`
    /// already exists in the database the `stack_size` and `last_grant` columns will be updated
    ///
    /// ## Argumnets
    /// * `db`              - The database connection
    /// * `user`            - The user this item belongs to
    /// * `definition_name` - The name of the item definition
    /// * `stack_size`      - The stack size to use / add for the item
    /// * `capacity`        - The stack max capacity if the definition defines one
    pub async fn add_item<'db, C>(
        db: &'db C,
        user: &User,
        definition_name: ItemName,
        stack_size: u32,
        capacity: Option<u32>,
    ) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        let now = Utc::now();

        // Upsert the inventory item
        Entity::insert(ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            definition_name: Set(definition_name),
            stack_size: Set(stack_size),
            instance_attributes: Set(SeaGenericMap::default()),
            created: Set(now),
            last_grant: Set(now),
            earned_by: Set("granted".to_string()),
            ..Default::default()
        })
        .on_conflict(
            // Update the value column if a key already exists
            OnConflict::columns([Column::UserId, Column::DefinitionName])
                .value(
                    Column::StackSize,
                    // Add the stack size but don't add above the capacity.
                    //
                    // The query below adds the stack size without surpassing
                    // the maximum capacity value
                    Expr::cust_with_values(
                        "(SELECT MIN(`stack_size` + ?, ?))",
                        [stack_size, capacity.unwrap_or(u32::MAX)],
                    ),
                )
                // Update the last granted column
                .update_column(Column::LastGrant)
                .to_owned(),
        )
        .exec(db)
        .await?;

        // Find the item that was updated or inserted
        let item = Entity::find()
            .filter(
                Column::UserId
                    .eq(user.id)
                    .and(Column::DefinitionName.eq(definition_name)),
            )
            .one(db)
            .await?;

        item.ok_or(DbErr::RecordNotInserted)
    }

    ///Sets the stack size of the item to `stack_size` if `stack_size` is zero
    /// then the item will be deleted
    pub async fn set_stack_size<C>(self, db: &C, stack_size: u32) -> DbResult<()>
    where
        C: ConnectionTrait,
    {
        // Remove empty stacks
        if stack_size == 0 {
            self.delete(db).await?;
            return Ok(());
        }

        // Update the model
        let mut model = self.into_active_model();
        model.stack_size = Set(stack_size);
        _ = model.update(db).await?;

        Ok(())
    }

    pub fn update_seen<'db, C>(
        db: &'db C,
        user: &User,
        list: Vec<ItemId>,
    ) -> impl Future<Output = DbResult<UpdateResult>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        // Updates all the matching items seen state
        Entity::update_many()
            .col_expr(Column::Seen, Expr::value(true))
            .filter(Column::Id.is_in(list).and(Column::UserId.eq(user.id)))
            .exec(db)
    }

    pub fn get_all_items<'db, C>(
        db: &'db C,
        user: &User,
    ) -> impl Future<Output = DbResult<Vec<InventoryItem>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db)
    }

    pub fn get_items<'db, C>(
        db: &'db C,
        user: &User,
        ids: Vec<ItemId>,
    ) -> impl Future<Output = DbResult<Vec<InventoryItem>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::Id.is_in(ids))
            .all(db)
    }

    /// Finds an item from the users collection of items with a matching `id`
    pub fn get<'db, C>(
        db: &'db C,
        user: &User,
        id: ItemId,
    ) -> impl Future<Output = DbResult<Option<InventoryItem>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).filter(Column::Id.eq(id)).one(db)
    }

    /// Finds a item with a matching definition `name` within the users
    /// collection of items
    pub fn get_by_name<'db, C>(
        db: &'db C,
        user: &User,
        name: ItemName,
    ) -> impl Future<Output = DbResult<Option<InventoryItem>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::DefinitionName.eq(name))
            .one(db)
    }
    /// Finds all items with a defintiion name in the collection of `names` that
    /// are within the user collection of items
    pub fn all_by_names<'db, C>(
        db: &'db C,
        user: &User,
        names: Vec<ItemName>,
    ) -> impl Future<Output = DbResult<Vec<InventoryItem>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::DefinitionName.is_in(names))
            .all(db)
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
