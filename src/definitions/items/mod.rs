use super::shared::CustomAttributes;
use crate::{
    database::entity::{InventoryItem, User, inventory_items::ItemId},
    definitions::{
        characters::acquire_item_character,
        classes::Classes,
        i18n::{I18n, I18nDescription, I18nName, Localized},
        level_tables::LevelTables,
    },
};
use anyhow::{Context, anyhow};
use category::{BaseCategory, Category};
use log::debug;
use rarity::ItemRarity;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none};
use std::{collections::HashMap, sync::OnceLock};
use uuid::{Uuid, uuid};

pub mod category;
pub mod link;
pub mod rarity;

/// Type of the name for items, names are [Uuid]s with some exceptions (Thanks EA)
pub type ItemName = Uuid;

/// Item definitions (628)
const INVENTORY_DEFINITIONS: &str = include_str!("../../resources/data/inventoryDefinitions.json");

/// Adds the collection of default items and characters to the
/// provided user
pub async fn create_default_items<C>(db: &C, user: &User) -> anyhow::Result<()>
where
    C: ConnectionTrait + Send,
{
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

        InventoryItem::add_item(db, user, definition.name, 1, definition.capacity)
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

/// Collection of [ItemDefinition]s with a lookup index based
/// on the [ItemName]s
pub struct Items {
    /// The underlying collection of [ItemDefinition]s
    values: Vec<ItemDefinition>,
    /// Lookup map for finding the index of a [ItemDefinition] based on its [ItemName]
    lookup_by_name: HashMap<ItemName, usize>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<Items> = OnceLock::new();

impl Items {
    /// Gets a static reference to the global [Items] collection
    pub fn get() -> &'static Items {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let mut values: Vec<ItemDefinition> = serde_json::from_str(INVENTORY_DEFINITIONS)
            .context("Failed to load inventory definitions")?;

        values.localize(I18n::get());

        debug!("Loaded {} item definition(s)", values.len());

        // Create the by name lookup table
        let lookup_by_name: HashMap<ItemName, usize> = values
            .iter()
            .enumerate()
            .map(|(index, definition)| (definition.name, index))
            .collect();

        Ok(Self {
            values,
            lookup_by_name,
        })
    }

    /// Collects a list of [ItemName] for any items that these item
    /// definitions may depend on, these items are then looked up in the
    /// the user inventory before generating rewards in order to check
    /// rules like "Must have A tier before we can drop B tier"
    pub fn droppable_required_names(&self) -> Vec<ItemName> {
        self.values
            .iter()
            // Only include droppable items
            .filter(|item| item.is_droppable())
            // Use unlock definition
            .filter_map(|item| item.unlock_definition.as_ref())
            .copied()
            .collect()
    }

    /// Collect all items that are droppable and have met the conditions
    /// for being dropped
    pub fn droppable_items(&self, owned_items: &[InventoryItem]) -> Vec<&ItemDefinition> {
        self.values
            .iter()
            .filter(|item| item.is_droppable() && item.is_conditions_met(self, owned_items))
            .collect()
    }

    /// Returns a slice to all the [ItemDefinition]s in this collection
    pub fn all(&self) -> &[ItemDefinition] {
        &self.values
    }

    /// Attempts to lookup an [ItemDefinition] by `names`
    pub fn by_name(&self, name: &ItemName) -> Option<&ItemDefinition> {
        let index = *self.lookup_by_name.get(name)?;
        self.values.get(index)
    }

    pub fn collect_by_name<'a, I: IntoIterator<Item = &'a Uuid>>(
        &self,
        iterator: I,
    ) -> Vec<ItemDefinition> {
        iterator
            .into_iter()
            .filter_map(|item| self.by_name(item))
            .cloned()
            .collect()
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefinition {
    /// Name of the item
    pub name: ItemName,

    /// Custom attributes associated with the item
    pub custom_attributes: CustomAttributes,

    /// Category the item falls under
    pub category: Category,

    /// Specifies other categories of items that can be attached to this
    /// item. Usually only used to specify weapon mod types on weapons.
    ///
    /// Other items leave an empty list
    pub attachable_categories: Vec<Category>,

    /// Rarity of the item
    pub rarity: Option<ItemRarity>,

    /// The maximum allowed capacity for this item within a players inventory
    #[serde(rename = "cap")]
    pub capacity: Option<u32>,

    /// Whether the item is consumable
    pub consumable: Option<bool>,
    /// Whether the item can be dropped from store rewards
    pub droppable: Option<bool>,
    /// Whether the item can be deleted
    pub deletable: Option<bool>,

    /// Specified if this item requires another item having reached its
    /// capacity before this item can be dropped.
    ///
    /// TODO: This field needs to be handled in store rewards
    /// Name of definition that this item depends on
    /// (Requires the item to reach its capacity before it can be dropped)
    /// TODO: Handle this when doing store rewards
    pub unlock_definition: Option<ItemName>,

    /// Activity events that should be created when various events are
    /// triggered around this item.
    ///
    /// Only present when the definitions are loaded for strike team missions?
    #[serde(flatten)]
    pub events: ItemEvents,

    /// TODO: I can't seem to find this field..? why have I added it..?
    pub restrictions: Option<String>,

    /// The default namespace this item belongs to
    pub default_namespace: InventoryNamespace,

    /// Not sure the use of this field, seems to always be `null`
    #[serialize_always]
    pub secret: Option<Value>,

    /// Localized item name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized item description
    #[serde(flatten)]
    pub i18n_description: Option<I18nDescription>,
}

impl Localized for ItemDefinition {
    fn localize(&mut self, i18n: &super::i18n::I18n) {
        self.i18n_name.localize(i18n);
        if let Some(i18n_description) = &mut self.i18n_description {
            i18n_description.localize(i18n);
        }
    }
}

impl ItemDefinition {
    #[inline]
    pub fn is_consumable(&self) -> bool {
        self.consumable.unwrap_or_default()
    }

    #[inline]
    pub fn is_droppable(&self) -> bool {
        self.droppable.unwrap_or_default()
    }

    #[inline]
    pub fn is_deletable(&self) -> bool {
        self.deletable.unwrap_or_default()
    }

    /// Get the parent "unlock definition" if one is present
    ///
    /// (This is the item that must be unlocked before this can be locked)
    fn unlock_definition<'def>(&self, defs: &'def Items) -> Option<&'def ItemDefinition> {
        self.unlock_definition
            .as_ref()
            .and_then(|unlock_def_name| defs.by_name(unlock_def_name))
    }

    /// Get this item from the list of owned items if present
    fn get_owned_item<'owned>(
        &self,
        owned_items: &'owned [InventoryItem],
    ) -> Option<&'owned InventoryItem> {
        owned_items
            .iter()
            .find(|item| item.definition_name == self.name)
    }

    /// Checks if the "unlock_definition" of this item definition is met
    fn is_unlock_definition_met(&self, owned_items: &[InventoryItem]) -> bool {
        self.get_owned_item(owned_items)
            // Ensure we also have the required capacity if present
            .is_some_and(|owned_item| {
                self.capacity
                    .is_none_or(|required_capacity| owned_item.stack_size >= required_capacity)
            })
    }

    /// Checks if the item drop conditions are met, recursively checks parent items
    /// to ensure the parent unlock conditions are met
    pub fn is_conditions_met(&self, defs: &Items, owned_items: &[InventoryItem]) -> bool {
        let unlock_def: &ItemDefinition = match self.unlock_definition(defs) {
            Some(value) => value,
            // No unlocking requirement
            None => return true,
        };

        unlock_def.is_unlock_definition_met(owned_items)
        // Ensure any parent definitions are also unlocked
            && unlock_def.is_conditions_met(defs, owned_items)
    }
}

/// Activity events that should be created when
/// different things happen to the item
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEvents {
    /// Activity event that should be created when the item is consumed
    pub on_consume: Option<Vec<Value>>,
    /// Activity event that should be created when the item is added
    pub on_add: Option<Vec<Value>>,
    /// Activity event that should be created when the item is removed
    pub on_remove: Option<Vec<Value>>,
}

/// Structure for tracking a change in stack size
/// for a specific item
#[derive(Debug)]
#[allow(unused)]
pub struct ItemChanged {
    /// ID of the item
    pub item_id: ItemId,
    /// The previous stack size of the item
    pub prev_stack_size: u32,
    /// The new stack size of the item
    pub stack_size: u32,
}

/// Known namespaces for the game
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InventoryNamespace {
    /// Default namespace
    Default,
    /// Strike team related namespace
    StrikeTeams,
    /// Blank namespace
    #[serde(rename = "")]
    None,
}

#[cfg(test)]
mod test {
    use super::Items;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = Items::load().unwrap();
    }
}
