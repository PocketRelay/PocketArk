use super::i18n::{I18n, I18nDescription, I18nName, Localized};
use super::shared::CustomAttributes;
use anyhow::Context;
use category::Category;
use log::debug;
use rarity::ItemRarity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none};
use std::{collections::HashMap, sync::OnceLock};
use uuid::Uuid;

pub mod category;
pub mod link;
pub mod rarity;

/// Type of the name for items, names are [Uuid]s with some exceptions (Thanks EA)
pub type ItemName = Uuid;

/// Item definitions (628)
const INVENTORY_DEFINITIONS: &str = include_str!("../../resources/data/inventoryDefinitions.json");

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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
    pub fn unlock_definition<'def>(&self, defs: &'def Items) -> Option<&'def ItemDefinition> {
        self.unlock_definition
            .as_ref()
            .and_then(|unlock_def_name| defs.by_name(unlock_def_name))
    }
}

/// Activity events that should be created when
/// different things happen to the item
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub item_id: Uuid,
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
