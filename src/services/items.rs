use crate::{
    database::{
        DbTransaction,
        dto::{
            inventory_items::{CreateInventoryItemDto, InventoryItemDto},
            users::UserDto,
        },
        repositories::inventory_items::InventoryItemsRepository,
    },
    definitions::{
        classes::Classes,
        items::{
            ItemDefinition, Items,
            category::{BaseCategory, Category},
        },
        level_tables::LevelTables,
    },
    services::characters::acquire_item_character,
};
use anyhow::anyhow;
use chrono::Utc;
use std::ops::DerefMut;
use uuid::uuid;

/// Adds the collection of default items and characters to the
/// provided user
pub async fn create_default_items(
    db: &mut DbTransaction<'_>,
    user: &UserDto,
) -> anyhow::Result<()> {
    let item_definitions = Items::get();
    let classes = Classes::get();
    let level_tables = LevelTables::get();

    // Create models from initial item defs
    let ids = [
        uuid!("af3a2cf0-dff7-4ca8-9199-73ce546c3e7b"), // HUMAN MALE SOLDIER
        uuid!("79f3511c-55da-67f0-5002-359c370015d8"), // HUMAN FEMALE SOLDIER
        uuid!("a3960123-3625-4126-82e4-1f9a127d33aa"), // HUMAN MALE ENGINEER
        uuid!("c756c741-1bc8-47a8-9f35-b7ca943ba034"), // HUMAN FEMALE ENGINEER
        uuid!("baae0381-8690-4097-ae6d-0c16473519b4"), // HUMAN MALE SENTINEL
        uuid!("319ffe5d-f8fb-4217-bd2f-2e8af4f53fc8"), // HUMAN FEMALE SENTINEL
        uuid!("7fd30824-e20c-473e-b906-f4f30ebc4bb0"), // HUMAN MALE VANGUARD
        uuid!("96fa16c5-9f2b-46f8-a491-a4b0a24a1089"), // HUMAN FEMALE VANGUARD
        uuid!("34aeef66-a030-445e-98e2-1513c0c78df4"), // HUMAN MALE INFILTRATOR
        uuid!("cae8a2f3-fdaf-471c-9391-c29f6d4308c3"), // HUMAN FEMALE INFILTRATOR
        uuid!("e4357633-93bc-4596-99c3-4cc0a49b2277"), // HUMAN MALE ADEPT
        uuid!("e2f76cf1-4b42-4dba-9751-f2add5c3f654"), // HUMAN FEMALE ADEPT
        uuid!("4ccc7f54-791c-4b66-954b-a0bd6496f210"), // M-3 PREDATOR
        uuid!("d5bf2213-d2d2-f892-7310-c39a15fb2ef3"), // M-8 AVENGER
        uuid!("38e07595-764b-4d9c-b466-f26c7c416860"), // VIPER
        uuid!("ca7d0f24-fc19-4a78-9d25-9c84eb01e3a5"), // M-23 KATANA
    ];

    for item in ids {
        let definition = item_definitions
            .by_name(&item)
            .ok_or(anyhow!("Missing default item '{item}'"))?;

        InventoryItemsRepository::add_item(
            db.deref_mut(),
            CreateInventoryItemDto {
                user_id: user.id,
                definition_name: definition.name,
                stack_size: 1,
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
