use crate::{
    database::entity::{inventory_items::ItemId, Character, Currency, InventoryItem, User},
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            inventory::{
                ConsumeRequest, InventoryRequestQuery, InventoryResponse, InventorySeenRequest,
                ItemDefinitionsResponse,
            },
            HttpResult, RawHttpError,
        },
    },
    services::{
        activity::{ActivityError, ActivityEvent, ActivityName, ActivityResult, ActivityService},
        items::{
            pack::{ItemReward, RewardCollection},
            BaseCategory, Category, ItemChanged, ItemDefinition, ItemName, ItemNamespace,
            ItemsService,
        },
    },
    state::App,
};
use axum::{extract::Query, Extension, Json};
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};
use thiserror::Error;

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory(
    Query(query): Query<InventoryRequestQuery>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<InventoryResponse> {
    let services = App::services();
    let mut items = InventoryItem::get_all_items(&db, &user).await?;

    // TODO: Possibly store namespace with item itself then only query that namespace directly
    if let Some(namespace) = query.namespace {
        if !matches!(namespace, ItemNamespace::None | ItemNamespace::Default) {
            // Remove items that aren't in the same namespace
            items.retain(|item| {
                services
                    .items
                    .items
                    .by_name(&item.definition_name)
                    .is_some_and(|def| def.default_namespace.eq(&namespace))
            });
        }
    }

    let definitions = if query.include_definitions {
        let defs = items
            .iter()
            .filter_map(|item| services.items.items.by_name(&item.definition_name))
            .collect();
        Some(defs)
    } else {
        None
    };

    Ok(Json(InventoryResponse { items, definitions }))
}

/// GET /inventory/definitions
///
/// Obtains the definitions for all the inventory items this includes things
/// like lootboxes, characters, weapons, etc.
pub async fn get_definitions() -> Json<ItemDefinitionsResponse> {
    let services = App::services();
    let list: &'static [ItemDefinition] = services.items.items.all();
    Json(ItemDefinitionsResponse {
        total_count: list.len(),
        list,
    })
}

/// PUT /inventory/seen
///
/// Updates the seen status of a list of inventory item IDs
pub async fn update_inventory_seen(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<InventorySeenRequest>,
) -> Result<StatusCode, RawHttpError> {
    debug!("Inventory seen change requested: {:?}", req);

    // Updates all the matching items seen state
    InventoryItem::update_seen(&db, &user, req.list).await?;

    // TODO: Actual database call to update the seen status
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Error)]
pub enum InventoryError {
    /// User doesn't own the item they tried to consume
    #[error("The user does not own the item.")]
    NotOwned,

    /// User doesn't own enough of the item
    #[error("Not enough of owned item")]
    NotEnough,

    /// Tried to consume a non-consumable item
    #[error("Item not consumable")]
    NotConsumable,

    /// Internal server error because item definition was missing
    #[error("Item missing definition")]
    MissingDefinition,

    /// Database error occurred
    #[error("Server error")]
    Database(#[from] DbErr),

    /// Error processing the activity
    #[error(transparent)]
    Activity(#[from] ActivityError),
}

/// Attempts to consume the provided `count` of `item` from the inventory of `user`.
/// If the user has the item then the item definition will be returned
async fn consume_item<'def, C>(
    db: &C,
    user: &User,
    item: ItemId,
    count: u32,
    items_service: &'def ItemsService,
) -> Result<&'def ItemDefinition, InventoryError>
where
    C: ConnectionTrait + Send,
{
    let mut item = InventoryItem::get(db, user, item)
        .await?
        // User doesn't own the item
        .ok_or(InventoryError::NotOwned)?;

    let definition: &'def ItemDefinition = items_service
        .items
        .by_name(&item.definition_name)
        .ok_or(InventoryError::MissingDefinition)?;

    // Ensure the item can be consumed
    if !definition.is_consumable() {
        return Err(InventoryError::NotConsumable);
    }

    // Sanity check incase the item exists in the DB even after becoming empty
    if item.stack_size < count {
        return Err(InventoryError::NotEnough);
    }

    let new_stack_size = item.stack_size - count;

    // Decrease the stack size
    item.set_stack_size(db, new_stack_size).await;

    Ok(definition)
}

/// POST /inventory/consume
///
/// Consumes an item from the inventory providing details about the changes to
/// the inventory. Used when lootboxes are opened and when consumables are used
/// within the game.
pub async fn consume_inventory(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<ConsumeRequest>,
) -> Result<Json<ActivityResult>, RawHttpError> {
    const CONSUME_COUNT: u32 = 1;

    debug!("Consume inventory items: {:?}", req);

    let services = App::services();
    let items_service = &services.items;

    let result: ActivityResult = db
        .transaction(|db| {
            Box::pin(async move {
                let mut events: Vec<ActivityEvent> = Vec::with_capacity(req.items.len());

                // Create the consumption event for each item
                for target in req.items {
                    let item_id = target.item_id;

                    // Attempt to consume the item
                    let item_definition =
                        consume_item(db, &user, item_id, CONSUME_COUNT, items_service).await?;

                    // Create the activity event
                    let event = ActivityEvent::new(ActivityName::ItemConsumed)
                        .with_attribute("category", item_definition.category.to_string())
                        .with_attribute("definitionName", item_definition.name)
                        .with_attribute("count", CONSUME_COUNT);

                    events.push(event);
                }

                // Process the event
                ActivityService::process_events(db, &user, events)
                    .await
                    .map_err(InventoryError::Activity)
            })
        })
        .await?;

    Ok(Json(result))
}

impl From<InventoryError> for RawHttpError {
    fn from(value: InventoryError) -> Self {
        let reason = value.to_string();
        let status = match value {
            InventoryError::NotOwned => StatusCode::NOT_FOUND,
            InventoryError::NotConsumable => StatusCode::BAD_REQUEST,
            InventoryError::NotEnough => StatusCode::CONFLICT,
            InventoryError::Database(_)
            | InventoryError::Activity(_)
            | InventoryError::MissingDefinition => StatusCode::INTERNAL_SERVER_ERROR,
        };

        RawHttpError::new_owned(reason, status)
    }
}
