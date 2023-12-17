use crate::{
    database::entity::{Character, Currency, InventoryItem},
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
        items::{
            pack::{ItemReward, RewardCollection},
            BaseCategory, Category, ItemChanged, ItemDefinition, ItemNamespace,
        },
    },
    state::App,
};
use axum::{extract::Query, Extension, Json};
use hyper::StatusCode;
use log::{debug, error, warn};
use rand::{rngs::StdRng, SeedableRng};
use sea_orm::{DatabaseConnection, DatabaseTransaction};

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
    // TODO: Database transaction to rollback changes on consume failure

    debug!("Consume inventory items: {:?}", req);

    let services = App::services();
    let items_service = &services.items;

    // Obtain the items and definitions that the user owns
    let owned_items: Vec<(InventoryItem, &'static ItemDefinition)> =
        InventoryItem::get_all_items(&db, &user)
            .await?
            .into_iter()
            .filter_map(|value| {
                let definition = items_service.items.by_name(&value.definition_name)?;
                Some((value, definition))
            })
            .collect();

    // List of changed item stack sizes
    let mut items_changed: Vec<ItemChanged> = Vec::new();
    // Collection of rewards
    let mut rewards: RewardCollection = RewardCollection::default();

    for target in req.items {
        // Find the owned item to consume
        let (item, definition) = owned_items
            .iter()
            .find(|(item, _)| item.id.eq(&target.item_id))
            .ok_or(HttpError::new(
                "The user does not own the item.",
                StatusCode::NOT_FOUND,
            ))?;

        // Ignore items that arent consumable
        if !definition.is_consumable() {
            return Err(HttpError::new(
                "Item not consumable",
                StatusCode::BAD_REQUEST,
            ));
        }

        // Obtain the base category of the item
        let base_category = match &definition.category {
            Category::Base(base) => *base,
            Category::Sub(sub) => sub.0,
        };

        match base_category {
            BaseCategory::ItemPack => {
                let pack = items_service
                    .packs
                    .by_name(&definition.name)
                    .ok_or_else(|| {
                        warn!(
                            "Don't know how to handle item pack: {} ({:?})",
                            &definition.name,
                            &definition.locale.name()
                        );
                        HttpError::new("Pack item not implemented", StatusCode::NOT_IMPLEMENTED)
                    })?;

                let mut rng = StdRng::from_entropy();

                pack.generate_rewards(&mut rng, &items_service.items, &owned_items, &mut rewards)
                    .map_err(|err| {
                        error!("Failed to grant pack items: {} {}", &pack.name, err);
                        HttpError::new(
                            "Failed to grant pack items",
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    })?;

                // TODO: Pack consumed activity
            }

            BaseCategory::ApexPoints => {
                // TODO: Apex point awards
            }
            BaseCategory::StrikeTeamReward => {
                // TODO: Strike team rewards
            }
            BaseCategory::Consumable => {}
            BaseCategory::Boosters => {}
            BaseCategory::CapacityUpgrade => {}

            _ => {}
        }

        // Take 1 from the item we just consumed
        items_changed.push(ItemChanged {
            item_id: item.id,
            prev_stack_size: item.stack_size,
            stack_size: item.stack_size.saturating_sub(1),
        });
    }

    // Process item changes
    for (item, definition) in owned_items {
        let change = items_changed
            .iter()
            .find(|value| value.item_id.eq(&item.id));
        if let Some(change) = change {
            debug!(
                "Consumed item stack size {} ({}) new stack size: x{}",
                item.id,
                definition.locale.name(),
                change.stack_size,
            );
            item.set_stack_size(&db, change.stack_size).await?;
        }
    }

    let rewards = rewards.rewards;
    let mut earned: Vec<InventoryItem> = Vec::with_capacity(rewards.len());
    let mut definitions: Vec<&'static ItemDefinition> = Vec::with_capacity(rewards.len());

    for ItemReward {
        definition,
        stack_size,
    } in rewards
    {
        debug!(
            "Item reward {} x{} ({:?} to {}",
            definition.name,
            stack_size,
            definition.locale.name(),
            user.username
        );

        let mut item =
            InventoryItem::add_item(&db, &user, definition.name, stack_size, definition.capacity)
                .await?;

        // Update the returning item stack size to the correct size
        // (Response should be the amount earned *not* total amount)
        item.stack_size = stack_size;

        // Handle character creation if the item is a character item
        if definition
            .category
            .is_within(&Category::Base(BaseCategory::Characters))
        {
            let services = App::services();
            Character::create_from_item(&db, &services.character, &user, &definition.name).await?;
        }

        earned.push(item);
        definitions.push(definition);
    }

    let currencies = Currency::all(&db, &user).await?;

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
