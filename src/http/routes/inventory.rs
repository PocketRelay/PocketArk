use axum::Json;
use hyper::StatusCode;
use log::debug;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Value,
};
use serde_json::Map;
use uuid::Uuid;

use crate::{
    database::entity::{inventory_items, InventoryItem, InventoryItemEntity},
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
    state::App,
};

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory(Auth(user): Auth) -> Result<Json<InventoryResponse>, HttpError> {
    let services = App::services();
    let db = App::database();

    let items = user.find_related(inventory_items::Entity).all(db).await?;

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
pub async fn update_inventory_seen(
    Auth(user): Auth,
    Json(req): Json<InventorySeenList>,
) -> Result<StatusCode, HttpError> {
    debug!("Inventory seen change requested: {:?}", req);

    let db = App::database();

    // Updates all the matching items seen state
    InventoryItemEntity::update_many()
        .col_expr(
            inventory_items::Column::Seen,
            Expr::value(Value::Bool(Some(true))),
        )
        .filter(inventory_items::Column::ItemId.is_in(req.list))
        .belongs_to(&user)
        .exec(db)
        .await?;

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

    let services = App::services();
    let db = App::database();

    // Collect all the items to be consumed
    let item_ids: Vec<Uuid> = req.items.into_iter().map(|value| value.item_id).collect();
    let items: Vec<InventoryItem> = user
        .find_related(inventory_items::Entity)
        .filter(inventory_items::Column::ItemId.is_in(item_ids))
        .all(db)
        .await?;

    let definitions: Vec<&'static ItemDefinition> = items
        .iter()
        .filter_map(|item| services.defs.inventory.map.get(&item.definition_name))
        .collect();

    // TODO: Ha: u32ndle pack opening, consuming etc
    let currencies = user.get_currencies(db).await?;

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
        prestige_progression_map: Map::new(),
        character_class_name: None,
    };

    Ok(Json(activity))
}
