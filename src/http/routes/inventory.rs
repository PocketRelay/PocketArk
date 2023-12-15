use crate::{
    database::entity::{Currency, InventoryItem},
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            inventory::{
                ConsumeRequest, InventoryRequestQuery, InventoryResponse, InventorySeenRequest,
                ItemDefinitionsResponse,
            },
            HttpError, HttpResult,
        },
    },
    services::{
        activity::{ActivityItemDetails, ActivityResult},
        items::{Category, GrantedItem, ItemChanged, ItemDefinition},
    },
    state::App,
};
use axum::{extract::Query, Extension, Json};
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use sea_orm::DatabaseConnection;

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

    if let Some(namespace) = query.namespace {
        if !namespace.is_empty() && namespace != "default" {
            // Remove items that aren't in the same namespace
            items.retain(|item| {
                services
                    .items
                    .by_name(&item.definition_name)
                    .is_some_and(|def| def.default_namespace.eq(&namespace))
            });
        }
    }

    let definitions = if query.include_definitions {
        let defs = items
            .iter()
            .filter_map(|item| services.items.by_name(&item.definition_name))
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
    let list: &'static [ItemDefinition] = services.items.defs();
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
) -> Result<StatusCode, HttpError> {
    debug!("Inventory seen change requested: {:?}", req);

    // Updates all the matching items seen state
    InventoryItem::update_seen(&db, &user, req.list).await?;

    // TODO: Actual database call to update the seen status
    Ok(StatusCode::NO_CONTENT)
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
) -> Result<Json<ActivityResult>, HttpError> {
    debug!("Consume inventory items: {:?}", req);

    let services = App::services();
    let items_service = &services.items;

    // Obtain the items and definitions that the user owns
    let owned_items: Vec<(InventoryItem, &'static ItemDefinition)> =
        InventoryItem::get_all_items(&db, &user)
            .await?
            .into_iter()
            .filter_map(|value| {
                let definition = items_service.by_name(&value.definition_name)?;
                Some((value, definition))
            })
            .collect();

    // List of changed item stack sizes
    let mut items_changed: Vec<ItemChanged> = Vec::new();
    // List of items granted to the user
    let mut items_granted: Vec<GrantedItem> = Vec::new();

    for target in req.items {
        // Find the owned item to consume
        let (item, definition) = owned_items
            .iter()
            .find(|(item, _)| item.item_id.eq(&target.item_id))
            .ok_or(HttpError::new(
                "The user does not own the item.",
                StatusCode::NOT_FOUND,
            ))?;

        // Ignore items that arent consumable
        if !definition.consumable.unwrap_or_default() {
            return Err(HttpError::new(
                "Item not consumable",
                StatusCode::BAD_REQUEST,
            ));
        }

        match definition.category.as_str() {
            Category::ITEM_PACK => {
                let pack = items_service
                    .pack_by_name(&definition.name)
                    .ok_or_else(|| {
                        warn!(
                            "Don't know how to handle item pack: {} ({:?})",
                            &definition.name,
                            &definition.locale.name()
                        );
                        HttpError::new("Pack item not implemented", StatusCode::NOT_IMPLEMENTED)
                    })?;

                let mut rng = StdRng::from_entropy();

                pack.grant_items(
                    &mut rng,
                    items_service.defs(),
                    &owned_items,
                    &mut items_granted,
                )
                .map_err(|err| {
                    error!("Failed to grant pack items: {} {}", &pack.name, err);
                    HttpError::new(
                        "Failed to grant pack items",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;

                // TODO: Pack consumed activity
            }
            Category::APEX_POINTS => {
                // TODO: Apex point awards
            }
            Category::STRIKE_TEAM_REWARD => {
                // TODO: Strike team rewards
            }
            Category::CONSUMABLE => {}
            Category::BOOSTERS => {}
            Category::CAPACITY_UPGRADE => {}
            _ => {}
        }

        // Take 1 from the item we just consumed
        items_changed.push(ItemChanged {
            item_id: item.item_id,
            prev_stack_size: item.stack_size,
            stack_size: item.stack_size.saturating_sub(1),
        });
    }

    // Process item changes
    for (item, definition) in owned_items {
        let change = items_changed
            .iter()
            .find(|value| value.item_id.eq(&item.item_id));
        if let Some(change) = change {
            debug!(
                "Consumed item stack size {} ({}) new stack size: x{}",
                item.item_id,
                definition.locale.name(),
                change.stack_size,
            );
            item.set_stack_size(&db, change.stack_size).await?;
        }
    }

    let (earned, definitions) = InventoryItem::grant_items(&db, &user, items_granted).await?;
    let currencies = Currency::get_from_user(&db, &user).await?;

    let activity = ActivityResult {
        currencies,
        items: ActivityItemDetails {
            earned,
            definitions,
        },
        ..Default::default()
    };

    Ok(Json(activity))
}
