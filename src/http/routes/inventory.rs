use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::debug;
use serde_json::Map;

use crate::http::models::{
    inventory::{InventoryConsumeResponse, InventorySeenList},
    store::Currency,
    HttpError, RawJson,
};

/// GET /inventory
pub async fn get_inventory() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Inventory_With_Definitions-1688700307239.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// Definitions for all the items
static INVENTORY_DEFINITIONS: &str = include_str!("../../resources/data/inventoryDefinitions.json");

/// GET /inventory/definitions
///
/// Obtains the definitions for all the inventory items this includes things
/// like lootboxes, characters, weapons, etc.
pub async fn get_definitions() -> RawJson {
    RawJson(INVENTORY_DEFINITIONS)
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
pub async fn consume_inventory() -> Result<Json<InventoryConsumeResponse>, HttpError> {
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

    let resp = InventoryConsumeResponse {
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

    Ok(Json(resp))
}
