use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use log::{debug, error};
use serde_json::Map;

use crate::{
    http::models::{
        inventory::{
            ActivityResult, InventoryConsumeRequest, InventoryDefinitions, InventoryItem,
            InventoryResponse, InventorySeenList, ItemDefinition,
        },
        store::Currency,
        HttpError,
    },
    state::App,
};

static PLACEHOLDER_INVENTORY: &str = include_str!("../../resources/data/placeholderInventory.json");

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory() -> Result<Json<InventoryResponse>, HttpError> {
    let services = App::services();
    let items: Vec<InventoryItem> = serde_json::from_str(PLACEHOLDER_INVENTORY).map_err(|e| {
        error!("Failed to load placeholder items: {}", e);
        HttpError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed to load placeholder items".to_string(),
            cause: None,
            stack_trace: None,
            trace_id: None,
        }
    })?;
    let definitions: Vec<&'static ItemDefinition> = items
        .iter()
        .filter_map(|item| services.defs.inventory.map.get(&item.definition_name))
        .collect();

    Ok(Json(InventoryResponse { items, definitions }))
}

/// GET /inventory/definitions
///
/// Obtains the definitions for all the inventory items this includes things
/// like lootboxes, characters, weapons, etc.
pub async fn get_definitions() -> Json<InventoryDefinitions> {
    let services = App::services();
    let list: &'static [ItemDefinition] = &services.defs.inventory.list;
    Json(InventoryDefinitions {
        total_count: list.len(),
        list,
    })
}

/// PUT /inventory/seen
///
/// Updates the seen status of a list of inventory item IDs
pub async fn update_inventory_seen(Json(req): Json<InventorySeenList>) -> Response {
    debug!("Inventory seen change requested: {:?}", req);
    // TODO: Actual database call to update the seen status
    StatusCode::NO_CONTENT.into_response()
}

/// POST /inventory/consume
///
/// Consumes an item from the inventory providing details about the changes to
/// the inventory. Used when lootboxes are opened and when consumables are used
/// within the game.
pub async fn consume_inventory(
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

    // TODO: Handle pack opening, consuming etc

    let balance = u32::MAX / 2;
    let currencies = vec![
        Currency {
            name: "MTXCurrency".to_string(),
            balance,
        },
        Currency {
            name: "GrindCurrency".to_string(),
            balance,
        },
        Currency {
            name: "MissionCurrency".to_string(),
            balance,
        },
    ];

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
        items_earned: vec![],
        item_definitions: vec![],
        entitlements_granted: vec![],
        prestige_progression_map: serde_json::Value::Object(Map::new()),
    };

    Ok(Json(activity))
}
