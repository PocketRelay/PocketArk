use chrono::Utc;
use openssl::stack;
use sea_orm::{
    entity::prelude::*,
    sea_query::Expr,
    ActiveValue::{NotSet, Set},
    IntoActiveModel,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::DbResult, http::models::inventory::ItemDefinition, services::items::Category,
    state::App,
};

use super::{Character, InventoryItem, User, ValueMap};

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
    /// Createsa a new item from the provided definition
    pub async fn create_item<C>(
        db: &C,
        user: &User,
        definition: &ItemDefinition,
        stack_size: u32,
    ) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        let now = Utc::now();
        let model = ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            item_id: Set(Uuid::new_v4()),
            definition_name: Set(definition.name.to_string()),
            stack_size: Set(stack_size),
            seen: Set(false),
            instance_attributes: Set(ValueMap::default()),
            created: Set(now),
            last_grant: Set(now),
            earned_by: Set("granted".to_string()),
            restricted: Set(false),
        }
        .insert(db)
        .await?;

        // Handle character creation
        if definition.category == Category::CHARACTERS {
            let uuid = Uuid::parse_str(&definition.name);
            if let Ok(uuid) = uuid {
                let services = App::services();

                Character::create_from_item(db, &services.defs, &user, uuid).await?;
            }
        }

        Ok(model)
    }

    /// Creates a new item if there are no matching item definitions in
    /// the inventory otherwise appends the stack size to the existing item
    pub async fn create_or_append<C>(
        db: &C,
        user: &User,
        definition: &ItemDefinition,
        stack_size: u32,
    ) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        if let Some(existing) = user
            .find_related(Entity)
            .filter(Column::DefinitionName.eq(&definition.name))
            .one(db)
            .await?
        {
            let capacity = definition.cap.as_ref().copied().unwrap_or(u32::MAX);
            let stack_size = existing.stack_size.saturating_add(stack_size).min(capacity);

            let mut model = existing.into_active_model();
            model.stack_size = Set(stack_size);
            model.last_grant = Set(Utc::now());
            model.update(db).await
        } else {
            Self::create_item(db, user, definition, stack_size).await
        }
    }

    /// Creates a new item if there are no matching item definitions in
    /// the inventory otherwise appends the stack size to the existing item
    pub async fn reduce_stack_size<C>(self, db: &C, amount: u32) -> DbResult<Option<Self>>
    where
        C: ConnectionTrait,
    {
        let stack_size = self.stack_size.saturating_sub(amount);
        if stack_size == 0 {
            self.delete(db).await?;
            Ok(None)
        } else {
            let mut model = self.into_active_model();
            model.stack_size = Set(stack_size);
            let model = model.update(db).await?;
            Ok(Some(model))
        }
    }

    pub async fn create_default<C>(db: &C, user: &User) -> DbResult<()>
    where
        C: ConnectionTrait + Send,
    {
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
        ];

        let services = App::services();

        for item in items {
            let def = match services.items.inventory.lookup(item) {
                Some(value) => value,
                None => continue,
            };

            Self::create_item(db, user, def, 1).await?;
        }

        Ok(())
    }

    pub async fn update_seen<C>(db: &C, user: &User, list: Vec<Uuid>) -> DbResult<()>
    where
        C: ConnectionTrait + Send,
    {
        // Updates all the matching items seen state
        Entity::update_many()
            .col_expr(Column::Seen, Expr::value(true))
            .filter(Column::ItemId.is_in(list).and(Column::UserId.eq(user.id)))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn get_all_items<C>(db: &C, user: &User) -> DbResult<Vec<InventoryItem>>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db).await
    }

    pub async fn get_items<C>(db: &C, user: &User, ids: Vec<Uuid>) -> DbResult<Vec<InventoryItem>>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::ItemId.is_in(ids))
            .all(db)
            .await
    }
}
