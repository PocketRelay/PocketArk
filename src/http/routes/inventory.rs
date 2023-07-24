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
    services::items::{Category, GrantedItem, ItemChanged},
    state::App,
};
use axum::Json;
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Map;

/// GET /inventory
///
/// Responds with a list of all the players inventory items along
/// with the definitions for the items
pub async fn get_inventory(Auth(user): Auth) -> Result<Json<InventoryResponse>, HttpError> {
    let db = App::database();
    let services = App::services();
    let items = InventoryItem::get_all_items(db, &user).await?;
    let definitions = items
        .iter()
        .filter_map(|item| services.items.by_name(&item.definition_name))
        .collect();

    Ok(Json(InventoryResponse { items, definitions }))
}

/// GET /inventory/definitions
///
/// Obtains the definitions for all the inventory items this includes things
/// like lootboxes, characters, weapons, etc.
pub async fn get_definitions() -> Json<InventoryDefinitions> {
    let services = App::services();
    let list: &'static [ItemDefinition] = services.items.defs();
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
    let mut granted: Vec<GrantedItem> = Vec::new();

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
            let pack = items_service.packs.get(&definition.name);
            if let Some(pack) = pack {
                let mut rng = StdRng::from_entropy();
                if let Err(err) =
                    pack.grant_items(&mut rng, items_service.defs(), &items, &mut granted)
                {
                    error!("Failed to grant pack items: {} {}", &definition.name, err);
                    return Err(HttpError::new(
                        "Failed to grant pack items",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                } else {
                    // TODO: Check item wasn't already changed

                    // Take 1 from the item we just consumed
                    items_changed.push(ItemChanged {
                        item_id: item.item_id,
                        prev_stack_size: item.stack_size,
                        stack_size: item.stack_size.saturating_sub(1),
                    });
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

    // TODO: Process item changes

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

    let currencies = Currency::get_from_user(db, &user).await?;

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
