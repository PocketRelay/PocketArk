use crate::{
    database::entity::{Currency, InventoryItem},
    http::{
        middleware::user::Auth,
        models::{
            inventory::{
                ConsumeRequest, InventoryRequestQuery, InventoryResponse, InventorySeenRequest,
                ItemDefinitionsResponse,
            },
            HttpError, HttpResult,
        },
    },
    services::{
        activity::ActivityResult,
        items::{Category, GrantedItem, ItemChanged, ItemDefinition, Pack},
    },
    state::App,
};
use axum::{extract::Query, Json};
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Map;

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory(
    Query(query): Query<InventoryRequestQuery>,
    Auth(user): Auth,
) -> HttpResult<InventoryResponse> {
    let db = App::database();
    let services = App::services();
    let mut items = InventoryItem::get_all_items(db, &user).await?;

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
    Json(req): Json<InventorySeenRequest>,
) -> Result<StatusCode, HttpError> {
    debug!("Inventory seen change requested: {:?}", req);

    let db = App::database();

    // Updates all the matching items seen state
    InventoryItem::update_seen(db, &user, req.list).await?;

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
    Json(req): Json<ConsumeRequest>,
) -> Result<Json<ActivityResult>, HttpError> {
    debug!("Consume inventory items: {:?}", req);

    let db = App::database();
    let services = App::services();
    let items_service = &services.items;

    let items: Vec<(InventoryItem, &'static ItemDefinition)> =
        InventoryItem::get_all_items(db, &user)
            .await?
            .into_iter()
            .filter_map(|value| {
                let definition = items_service.by_name(&value.definition_name)?;
                Some((value, definition))
            })
            .collect();

    let mut items_changed: Vec<ItemChanged> = Vec::new();
    let mut items_granted: Vec<GrantedItem> = Vec::new();

    for target in req.items {
        let (item, definition) = items
            .iter()
            .find(|(item, _)| item.item_id.eq(&target.item_id))
            .ok_or(HttpError::new(
                "The user does not own the item.",
                StatusCode::NOT_FOUND,
            ))?;

        if !definition.consumable.unwrap_or_default() {
            return Err(HttpError::new(
                "Item not consumable",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Item pack consumables
        if definition.category == Category::ITEM_PACK {
            let pack = items_service.packs.get(&definition.name).ok_or_else(|| {
                warn!(
                    "Don't know how to handle item pack: {} ({:?})",
                    &definition.name,
                    &definition.locale.name()
                );
                HttpError::new("Pack item not implemented", StatusCode::NOT_IMPLEMENTED)
            })?;

            consume_pack(
                item,
                pack,
                &mut items_changed,
                &mut items_granted,
                &items,
                items_service.defs(),
            )?;
        } else if definition.category == Category::CONSUMABLE {
            // TODO: consume the item
            items_changed.push(ItemChanged {
                item_id: item.item_id,
                prev_stack_size: item.stack_size,
                stack_size: item.stack_size.saturating_sub(1),
            });
        }
    }

    // Process item changes
    for (item, _) in items {
        let change = items_changed
            .iter()
            .find(|value| value.item_id.eq(&item.item_id));
        if let Some(change) = change {
            item.set_stack_size(db, change.stack_size).await?;
        }
    }

    let mut definitions: Vec<&'static ItemDefinition> = Vec::with_capacity(items_granted.len());
    let mut items_out: Vec<InventoryItem> = Vec::with_capacity(items_granted.len());

    for granted in items_granted {
        debug!(
            "Granted item {} x{} ({:?}",
            granted.defintion.name,
            granted.stack_size,
            granted.defintion.locale.name()
        );

        let mut item =
            InventoryItem::create_or_append(db, &user, granted.defintion, granted.stack_size)
                .await?;

        item.stack_size = granted.stack_size;

        debug!("Item stack size: {}", item.stack_size);

        items_out.push(item);
        definitions.push(granted.defintion);
    }

    let currencies = Currency::get_from_user(db, &user).await?;

    let activity = ActivityResult {
        currencies,
        items_earned: items_out,
        item_definitions: definitions,
        ..Default::default()
    };

    Ok(Json(activity))
}

fn consume_pack(
    item: &InventoryItem,
    pack: &Pack,
    items_changed: &mut Vec<ItemChanged>,
    items_granted: &mut Vec<GrantedItem>,
    items_owned: &[(InventoryItem, &'static ItemDefinition)],
    item_defs: &'static [ItemDefinition],
) -> Result<(), HttpError> {
    let mut rng = StdRng::from_entropy();

    pack.grant_items(&mut rng, item_defs, items_owned, items_granted)
        .map_err(|err| {
            error!("Failed to grant pack items: {} {}", &pack.name, err);
            HttpError::new(
                "Failed to grant pack items",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Take 1 from the item we just consumed
    items_changed.push(ItemChanged {
        item_id: item.item_id,
        prev_stack_size: item.stack_size,
        stack_size: item.stack_size.saturating_sub(1),
    });

    Ok(())
}
