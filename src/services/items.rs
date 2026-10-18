use crate::{resources::Resources, services::characters::acquire_item_character};
use anyhow::anyhow;
use chrono::Utc;
use pocket_ark_database::{
    DbTransaction,
    dto::{
        inventory_items::{CreateInventoryItemDto, InventoryItemDto},
        users::UserDto,
    },
    repositories::inventory_items::InventoryItemsRepository,
};
use pocket_ark_definitions::{
    classes::Classes,
    items::{
        ItemDefinition, Items,
        category::{BaseCategory, Category},
    },
    level_tables::LevelTables,
};
use serde::Deserialize;
use serde_with::skip_serializing_none;
use std::ops::DerefMut;
use uuid::Uuid;

#[skip_serializing_none]
#[derive(Debug, Deserialize)]
pub struct DefaultInventoryDefinition {
    pub items: Vec<DefaultInventoryDefinitionItem>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize)]
pub struct DefaultInventoryDefinitionItem {
    pub id: Uuid,
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
}

pub fn default_stack_size() -> u32 {
    1
}

/// Path to the default inventory JSON file resource
const USER_DEFAULT_INVENTORY_RESOURCE: &str = "userDefaultInventory.json";

/// Load the default inventory definition
pub async fn load_default_inventory_definition() -> anyhow::Result<DefaultInventoryDefinition> {
    let resource = Resources::get_resource_bytes(USER_DEFAULT_INVENTORY_RESOURCE).await?;
    let parsed = serde_json::from_slice(&resource)?;
    Ok(parsed)
}

/// Adds the collection of default items and characters to the
/// provided user
pub async fn create_default_items(
    item_definitions: &Items,
    classes: &Classes,
    level_tables: &LevelTables,
    db: &mut DbTransaction<'_>,
    user: &UserDto,
) -> anyhow::Result<()> {
    let defs = load_default_inventory_definition().await?;

    for item in defs.items {
        let definition = item_definitions
            .by_name(&item.id)
            .ok_or(anyhow!("Missing default item '{}'", item.id))?;

        InventoryItemsRepository::add_item(
            db.deref_mut(),
            CreateInventoryItemDto {
                user_id: user.id,
                definition_name: definition.name,
                stack_size: item.stack_size,
                capacity: definition.capacity,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        // Handle character creation if the item is a character item
        if definition
            .category
            .is_within(&Category::Base(BaseCategory::Characters))
        {
            acquire_item_character(db, user, &definition.name, classes, level_tables).await?;
        }
    }

    Ok(())
}

/// Get this item from the list of owned items if present
fn get_owned_item<'owned>(
    definition: &ItemDefinition,
    owned_items: &'owned [InventoryItemDto],
) -> Option<&'owned InventoryItemDto> {
    owned_items
        .iter()
        .find(|item| item.definition_name == definition.name)
}

/// Checks if the "unlock_definition" of this item definition is met
fn is_unlock_definition_met(definition: &ItemDefinition, owned_items: &[InventoryItemDto]) -> bool {
    get_owned_item(definition, owned_items)
        // Ensure we also have the required capacity if present
        .is_some_and(|owned_item| {
            definition
                .capacity
                .is_none_or(|required_capacity| owned_item.stack_size >= required_capacity)
        })
}

/// Checks if the item drop conditions are met, recursively checks parent items
/// to ensure the parent unlock conditions are met
pub fn is_conditions_met(
    definition: &ItemDefinition,
    defs: &Items,
    owned_items: &[InventoryItemDto],
) -> bool {
    let unlock_def: &ItemDefinition = match definition.unlock_definition(defs) {
        Some(value) => value,
        // No unlocking requirement
        None => return true,
    };

    is_unlock_definition_met(unlock_def, owned_items)
    // Ensure any parent definitions are also unlocked
        && is_conditions_met(unlock_def, defs, owned_items)
}

/// Get items that can be dropped from the items definition set based on the
/// current set of `owned_items` to enforce unlock conditions
pub fn get_droppable_items<'def>(
    items: &'def Items,
    owned_items: &[InventoryItemDto],
) -> Vec<&'def ItemDefinition> {
    items
        .all()
        .iter()
        .filter(|definition| {
            definition.is_droppable() && is_conditions_met(definition, items, owned_items)
        })
        .collect()
}
