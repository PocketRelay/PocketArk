use std::{
    borrow::Cow,
    fmt::{Display, Write},
    num::ParseIntError,
    str::FromStr,
};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::utils::models::LocaleNameWithDesc;

use super::pack::Packs;

pub const INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

pub struct ItemsService {
    pub items: ItemDefinitions,
    pub packs: Packs,
}

impl ItemsService {
    pub fn new() -> Self {
        let items = ItemDefinitions::from_str(INVENTORY_DEFINITIONS).unwrap();
        let packs = Packs::new();
        Self { items, packs }
    }
}

/// Type of the name for items, names are [Uuid]s with some exceptions (Thanks EA)
pub type ItemName = Uuid;

/// Link to an item, contains the item category and [ItemName]
pub struct ItemLink(BaseCategory, ItemName);

/// Collection of [ItemDefinition]s with a lookup index based
/// on the [ItemName]s
pub struct ItemDefinitions {
    /// The underlying collection of [ItemDefinition]s
    values: Vec<ItemDefinition>,
    /// Lookup map for finding the index of a [ItemDefinition] based on its [ItemName]
    lookup_by_name: HashMap<ItemName, usize>,
}

impl ItemDefinitions {
    /// Creates a new collection of [ItemDefinition]s from the provided JSON
    /// array string `value`
    pub fn from_str(value: &str) -> serde_json::Result<Self> {
        let values: Vec<ItemDefinition> = serde_json::from_str(value)?;

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

    /// Returns a slice to all the [ItemDefinition]s in this collection
    pub fn all(&self) -> &[ItemDefinition] {
        &self.values
    }

    /// Attempts to lookup an [ItemDefinition] by `names`
    pub fn by_name(&self, name: &Uuid) -> Option<&ItemDefinition> {
        let index = *self.lookup_by_name.get(name)?;
        self.values.get(index)
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefinition {
    /// Name of the item
    pub name: ItemName,

    /// Localization information for the item
    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,

    /// Custom attributes associated with the item
    pub custom_attributes: HashMap<String, Value>,

    /// Category the item falls under
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub category: Category,

    /// Specifies other categories of items that can be attached to this
    /// item. Usually only used to specify weapon mod types on weapons.
    ///
    /// Other items leave an empty list
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub attachable_categories: Vec<Category>,

    /// Rarity of the item
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
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
    pub default_namespace: ItemNamespace,

    /// Not sure the use of this field, seems to always be `null`
    #[serialize_always]
    pub secret: Option<Value>,
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
}

/// Activity events that should be created when
/// different things happen to the item
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub enum ItemNamespace {
    /// Default namespace
    Default,
    /// Striketeam related namespace
    Striketeams,
    /// Blank namespace
    #[serde(rename = "")]
    None,
}

/// Item rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum ItemRarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    UltraRare = 3,
    /// Appears on some weapon mods, possibly hidden mods?
    Max = 4,
}

impl ItemRarity {
    /// Provides the weight to use for this rarity value
    /// (Lower rarity has a higher weight)
    pub const fn weight(&self) -> u32 {
        match self {
            ItemRarity::Common => 32,
            ItemRarity::Uncommon => 24,
            ItemRarity::Rare => 16,
            ItemRarity::UltraRare => 8,
            ItemRarity::Max => 1,
        }
    }
}

/// Represents an item category
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Category {
    /// Base category portion
    Base(BaseCategory),
    /// Sub category
    Sub(SubCategory),
}

impl Category {
    /// Checks if this category is apart of another category.
    ///
    /// If both sides are [Category::Sub] then a full equality check is done
    /// otherwise only the [BaseCategory] portion is checked
    pub fn is_within(&self, other: &Category) -> bool {
        match (self, other) {
            // Both sides are matching types (Full equality)
            (Self::Base(left), Self::Base(right)) => left.eq(right),
            (Self::Sub(left), Self::Sub(right)) => left.eq(right),

            // One side is base category (Partial equality)
            (Self::Base(left), Self::Sub(right)) => right.0.eq(left),
            (Self::Sub(left), Self::Base(right)) => left.0.eq(right),
        }
    }
}

/// Categories of items
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum BaseCategory {
    /// Items associated with characters
    Characters = 0,
    /// Weapon items
    Weapons = 1,
    /// Weapon mods
    WeaponMods = 2,
    /// Boosters such as "AMMO CAPACITY MOD I", "ASSAULT RIFLE RAIL AMP", "CRYO AMMO"
    Boosters = 3,
    // Consumable items such as "AMMO PACK", "COBTRA RPG", "REVIVE PACK"
    Consumable = 4,
    /// Equipment such as "ADAPTIVE WAR AMP", and "ASSAULT LOADOUT"
    Equipment = 5,
    /// Rewards from challenges
    ChallengeReward = 7,
    /// Non droppable rewards for apex points
    ApexPoints = 8,
    /// Upgrades for capacity such as "AMMO PACK CAPACITY INCREASE" and
    /// "CHARACTER RESPEC" items
    CapacityUpgrade = 9,
    /// Rewards from strike team missions (Loot boxes)
    StrikeTeamReward = 11,
    /// Item loot box packs
    ItemPack = 12,
    /// Specialized weapons
    WeaponsSpecialized = 13,
    /// Enhanced weapon mod variants
    WeaponModsEnhanced = 14,
}

/// Sub category within a [BaseCategory]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SubCategory(BaseCategory, String);

/// Weapon categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeaponCategory {
    AssaultRifle,
    Pistol,
    Shotgun,
    SniperRifle,
}

impl From<WeaponCategory> for String {
    fn from(value: WeaponCategory) -> Self {
        value.to_string()
    }
}

impl SubCategory {
    #[inline]
    fn new<V>(base: BaseCategory, value: V) -> Self
    where
        V: Into<String>,
    {
        Self(base, value.into())
    }

    /// Creates a [SubCategory] that can represent any item within a category
    pub fn all(category: BaseCategory) -> Self {
        // Empty string denotes any sub category
        const ALL: &str = "";

        Self::new(category, ALL)
    }
}

impl Display for ItemRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Base(value) => Display::fmt(value, f),
            Category::Sub(value) => Display::fmt(value, f),
        }
    }
}

impl Display for BaseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
    }
}

impl Display for SubCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)?;
        f.write_char(':')?;
        Display::fmt(&self.1, f)
    }
}

impl Display for WeaponCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            WeaponCategory::AssaultRifle => "AssaultRifle",
            WeaponCategory::Pistol => "Pistol",
            WeaponCategory::Shotgun => "Shotgun",
            WeaponCategory::SniperRifle => "SniperRifle",
        })
    }
}
/// Errors that can occur when parsing an [ItemLink]
#[derive(Debug, Error)]
pub enum ItemLinkError {
    /// Error parsing the category portion
    #[error(transparent)]
    Base(#[from] BaseCategoryError),
    /// Item name portion of the link is missing
    #[error("Item link missing item name")]
    MissingName,
    /// Error parsing the item name
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl FromStr for ItemLink {
    type Err = ItemLinkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, name) = s.split_once(':').ok_or(ItemLinkError::MissingName)?;
        let base: BaseCategory = base.parse()?;
        let name: ItemName = name.parse()?;

        Ok(Self(base, name))
    }
}

/// Errors that can occur when parsing a [Rarity] from string
#[derive(Debug, Error)]
pub enum RarityError {
    /// Error parsing integer value
    #[error(transparent)]
    Parse(#[from] ParseIntError),
    /// Error converting value
    #[error(transparent)]
    FromPrimitive(#[from] TryFromPrimitiveError<ItemRarity>),
}

impl FromStr for ItemRarity {
    type Err = RarityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        let value: ItemRarity = ItemRarity::try_from_primitive(value)?;
        Ok(value)
    }
}

/// Errors that can occur when parsing a [Category]
#[derive(Debug, Error)]
pub enum CategoryError {
    /// Failed to parse the base category portion
    #[error(transparent)]
    BaseCategory(#[from] BaseCategoryError),
    /// Category was empty
    #[error("Category was empty")]
    Empty,
}

impl FromStr for Category {
    type Err = CategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(CategoryError::Empty);
        }

        let (base, sub) = s
            .split_once(':')
            .map(|(left, right)| (left, Some(right)))
            .unwrap_or((s, None));

        let base: BaseCategory = base.parse()?;

        Ok(if let Some(sub) = sub {
            Self::Sub(SubCategory(base, sub.to_string()))
        } else {
            Self::Base(base)
        })
    }
}

/// Errors that can occur when parsing a [BaseCategory]
#[derive(Debug, Error)]
pub enum BaseCategoryError {
    /// Failed to parse the primitive value from string
    #[error(transparent)]
    Parse(#[from] ParseIntError),
    /// Failed to convert the primitive value
    #[error(transparent)]
    FromPrimitive(#[from] TryFromPrimitiveError<BaseCategory>),
}

impl FromStr for BaseCategory {
    type Err = BaseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        let value: BaseCategory = BaseCategory::try_from_primitive(value)?;
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::{ItemDefinition, INVENTORY_DEFINITIONS};

    #[test]
    fn deserialize_items() {
        let values: Vec<ItemDefinition> = serde_json::from_str(INVENTORY_DEFINITIONS).unwrap();
        let mut vars = HashSet::new();
        for value in &values {
            vars.insert(&value.name);
        }
        println!("{:?}", vars);
    }
}
