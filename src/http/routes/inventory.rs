use axum::Json;
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Map;
use uuid::Uuid;

use crate::{
    database::entity::{Currency, InventoryItem},
    http::{
        middleware::user::Auth,
        models::{
            inventory::{
                ActivityResult, InventoryConsumeRequest, InventoryDefinitions, InventoryResponse,
                InventorySeenList, ItemDefinition,
            },
            HttpError,
        },
    },
    services::{
        self,
        items::{Category, GrantedItem},
    },
    state::App,
};

pub fn get_item_definitions(items: &[InventoryItem]) -> Vec<&'static ItemDefinition> {
    let services = App::services();
    let defs = &services.items.inventory;

    items
        .iter()
        .filter_map(|item| defs.lookup(&item.definition_name))
        .collect()
}

pub fn get_item_definition(item: &String) -> Option<&'static ItemDefinition> {
    let services = App::services();
    let defs = &services.items.inventory;
    defs.lookup(item)
}

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory(Auth(user): Auth) -> Result<Json<InventoryResponse>, HttpError> {
    let db = App::database();
    let items = InventoryItem::get_all_items(db, &user).await?;
    let definitions = get_item_definitions(&items);

    Ok(Json(InventoryResponse { items, definitions }))
}

/// GET /inventory/definitions
///
/// Obtains the definitions for all the inventory items this includes things
/// like lootboxes, characters, weapons, etc.
pub async fn get_definitions() -> Json<InventoryDefinitions> {
    let services = App::services();
    let list: &'static [ItemDefinition] = services.items.inventory.list();
    Json(InventoryDefinitions {
        total_count: list.len(),
        list,
    })
}

/// PUT /inventory/seen
///
/// Updates the seen status of a list of inventory item IDs
pub async fn update_inventory_seen(
    Auth(user): Auth,
    Json(req): Json<InventorySeenList>,
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
    Json(req): Json<InventoryConsumeRequest>,
) -> Result<Json<ActivityResult>, HttpError> {
    debug!("Consume inventory items: {:?}", req);

    // Replace with actual database lookup
    let exists: bool = true;

    if !exists {
        return Err(HttpError {
            status: StatusCode::NOT_FOUND,
            reason: "The user does not own the item.".to_string(),
            cause: None,
            stack_trace: None,
            trace_id: None,
        });
    }

    let db = App::database();
    let services = App::services();

    let items_service = &services.items;

    // Collect all the items to be consumed
    let item_ids: Vec<Uuid> = req.items.into_iter().map(|value| value.item_id).collect();
    let items: Vec<InventoryItem> = InventoryItem::get_items(db, &user, item_ids).await?;

    let mut granted: Vec<GrantedItem> = Vec::new();

    for item in items {
        let definition = match services.items.inventory.lookup(&item.definition_name) {
            Some(value) => value,
            None => continue,
        };

        if !definition.consumable.unwrap_or_default() {
            return Err(HttpError::new(
                "Item not consumable",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Item pack consumables
        if definition.category == Category::ITEM_PACK {
            let pack = items_service.packs.get(&definition.name);
            if let Some(pack) = pack {
                let mut rng = StdRng::from_entropy();
                if let Err(err) =
                    pack.grant_items(&mut rng, items_service.inventory.list(), &mut granted)
                {
                    error!("Failed to grant pack items: {} {}", &definition.name, err);
                    return Err(HttpError::new(
                        "Failed to grant pack items",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                } else {
                    // Take 1 from the item we just consumed
                    item.reduce_stack_size(db, 1).await?;
                }
            } else {
                warn!(
                    "Don't know how to handle item pack: {} ({:?})",
                    &definition.name, &definition.loc_name
                );
                return Err(HttpError::new(
                    "Pack item not implemented",
                    StatusCode::NOT_IMPLEMENTED,
                ));
            }
        } else if definition.category == Category::CONSUMABLE {
            // TODO: consume the item
        }
    }

    // TODO: Ha: u32ndle pack opening, consuming etc

    let mut definitions: Vec<&'static ItemDefinition> = Vec::with_capacity(granted.len());
    let mut items_out: Vec<InventoryItem> = Vec::with_capacity(granted.len());

    for granted in granted {
        debug!(
            "Granted item {} x{} ({:?}",
            granted.defintion.name, granted.stack_size, granted.defintion.loc_name
        );

        let mut item =
            InventoryItem::create_or_append(db, &user, granted.defintion, granted.stack_size)
                .await?;

        item.stack_size = granted.stack_size;

        debug!("Item stack size: {}", item.stack_size);

        items_out.push(item);
        definitions.push(granted.defintion);
    }

    let currencies = Currency::get_from_user(&user, db).await?;

    let activity = ActivityResult {
        previous_xp: 0,
        xp: 0,
        xp_gained: 0,
        previous_level: 0,
        level: 0,
        level_up: false,
        challenges_updated_count: 0,
        challenges_completed_count: 0,
        challenges_updated: vec![],
        updated_challenge_ids: vec![],
        news_triggered: 0,
        currencies,
        currency_earned: vec![],
        items_earned: items_out,
        item_definitions: definitions,
        entitlements_granted: vec![],
        prestige_progression_map: Map::new(),
        character_class_name: None,
    };

    Ok(Json(activity))
}
