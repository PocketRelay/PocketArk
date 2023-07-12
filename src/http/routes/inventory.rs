use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::debug;

use crate::http::models::{inventory::InventorySeenList, RawJson};

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
pub async fn consume_inventory() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/PostInventoryConsume.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
