//! Inventory item database models
//!
//!
//! Note: when manually querying the database for the `definition_name` column, its stored as
//! a BLOB TEXT, so to query it you must prefix the string with "x" like the following query:
//! ```sql
//! SELECT `definition_name` FROM `inventory_items` WHERE `definition_name` = x'af3a2cf0dff74ca8919973ce546c3e7b'`
//! ```
//! (Don't include hyphens in the definition name)

use crate::{
    database::{
        entity::{Character, InventoryItem, User, ValueMap},
        DbResult,
    },
    services::{
        character::CharacterService,
        items::{pack::ItemReward, BaseCategory, Category, ItemDefinition, ItemName, ItemsService},
    },
    state::App,
};
use chrono::Utc;
use futures::Future;
use log::debug;
use sea_orm::{
    entity::prelude::*,
    sea_query::{Expr, OnConflict, Query, SimpleExpr},
    ActiveValue::{NotSet, Set},
    IntoActiveModel, UpdateResult,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::str::FromStr;
use uuid::uuid;

use super::users::UserId;

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
            instance_attributes: Set(ValueMap::default()),
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

    pub async fn create_default<C>(
        db: &C,
        user: &User,
        items_service: &ItemsService,
        characters: &CharacterService,
    ) -> DbResult<()>
    where
        C: ConnectionTrait + Send,
    {
        // Create models from initial item defs
        let items = [
            uuid!("af3a2cf0-dff7-4ca8-9199-73ce546c3e7b"), // HUMAN MALE SOLDIER
            uuid!("79f3511c-55da-67f0-5002-359c370015d8"), // HUMAN FEMALE SOLDIER
            uuid!("a3960123-3625-4126-82e4-1f9a127d33aa"), // HUMAN MALE ENGINEER
            uuid!("c756c741-1bc8-47a8-9f35-b7ca943ba034"), // HUMAN FEMALE ENGINEER
            uuid!("baae0381-8690-4097-ae6d-0c16473519b4"), // HUMAN MALE SENTINEL
            uuid!("319ffe5d-f8fb-4217-bd2f-2e8af4f53fc8"), // HUMAN FEMALE SENTINEL
            uuid!("7fd30824-e20c-473e-b906-f4f30ebc4bb0"), // HUMAN MALE VANGUARD
            uuid!("96fa16c5-9f2b-46f8-a491-a4b0a24a1089"), // HUMAN FEMALE VANGUARD
            uuid!("34aeef66-a030-445e-98e2-1513c0c78df4"), // HUMAN MALE INFILTRATOR
            uuid!("cae8a2f3-fdaf-471c-9391-c29f6d4308c3"), // HUMAN FEMALE INFILTRATOR
            uuid!("e4357633-93bc-4596-99c3-4cc0a49b2277"), // HUMAN MALE ADEPT
            uuid!("e2f76cf1-4b42-4dba-9751-f2add5c3f654"), // HUMAN FEMALE ADEPT
            uuid!("4ccc7f54-791c-4b66-954b-a0bd6496f210"), // M-3 PREDATOR
            uuid!("d5bf2213-d2d2-f892-7310-c39a15fb2ef3"), // M-8 AVENGER
            uuid!("38e07595-764b-4d9c-b466-f26c7c416860"), // VIPER
            uuid!("ca7d0f24-fc19-4a78-9d25-9c84eb01e3a5"), // M-23 KATANA
        ];

        for item in items {
            let definition = match items_service.items.by_name(&item) {
                Some(value) => value,
                None => continue,
            };

            Self::add_item(db, user, definition.name, 1, definition.capacity)
                .await
                .unwrap();

            // Handle character creation if the item is a character item
            if definition
                .category
                .is_within(&Category::Base(BaseCategory::Characters))
            {
                Character::create_from_item(db, characters, user, &definition.name).await?;
            }
        }

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
