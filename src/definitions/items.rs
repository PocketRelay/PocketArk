use crate::{
    database::entity::{inventory_items::ItemId, InventoryItem, User},
    definitions::{
        characters::acquire_item_character,
        classes::Classes,
        i18n::{I18nDescription, I18nName},
        level_tables::LevelTables,
    },
};
use anyhow::{anyhow, Context};
use log::debug;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none, DeserializeAs, DisplayFromStr};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    num::ParseIntError,
    str::FromStr,
    sync::OnceLock,
};
use thiserror::Error;
use uuid::{uuid, Uuid};

/// Item definitions (628)
const INVENTORY_DEFINITIONS: &str = include_str!("../resources/data/inventoryDefinitions.json");

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

/// Type of the name for items, names are [Uuid]s with some exceptions (Thanks EA)
pub type ItemName = Uuid;

/// Link to an item, contains the item category and [ItemName]
#[derive(Debug)]
pub struct ItemLink(pub BaseCategory, pub ItemName);

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
        let values: Vec<ItemDefinition> = serde_json::from_str(INVENTORY_DEFINITIONS)
            .context("Failed to load inventory definitions")?;

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

    /// Returns a slice to all the [ItemDefinition]s in this collection
    pub fn all(&self) -> &[ItemDefinition] {
        &self.values
    }

    /// Attempts to lookup an [ItemDefinition] by `names`
    pub fn by_name(&self, name: &ItemName) -> Option<&ItemDefinition> {
        let index = *self.lookup_by_name.get(name)?;
        self.values.get(index)
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefinition {
    /// Name of the item
    pub name: ItemName,

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

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Category {
    /// Retrieves the base category of this category
    pub fn base(&self) -> BaseCategory {
        match self {
            Self::Base(base) => *base,
            Self::Sub(sub) => sub.0,
        }
    }

    /// Checks if this category has a matching base category
    pub fn base_eq(&self, other: &BaseCategory) -> bool {
        match self {
            Self::Base(base) => base.eq(other),
            Self::Sub(sub) => sub.0.eq(other),
        }
    }

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
pub struct SubCategory(pub BaseCategory, pub String);

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

impl Display for ItemLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)?;
        f.write_char(':')?;
        Display::fmt(&self.1, f)
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

impl<'de> Deserialize<'de> for ItemLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}

impl Serialize for ItemLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
    use super::Items;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = Items::load().unwrap();
    }
}
