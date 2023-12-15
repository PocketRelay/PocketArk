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

use super::ItemEvents;

pub const INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

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
    pub rarity: Option<Rarity>,

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
    /// triggered around this item
    #[serde(flatten)]
    pub events: ItemEvents,

    /// TODO: I can't seem to find this field..? why have I added it..?
    pub restrictions: Option<String>,

    /// The default namespace this item should be placed under. Seems to
    /// be majority left blank
    pub default_namespace: String,

    /// Not sure the use of this field, seems to always be `null`
    #[serialize_always]
    pub secret: Option<Value>,
}

/// Item rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum Rarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    UltraRare = 3,
    /// Appears on some weapon mods, possibly hidden mods?
    Max = 4,
}

/// Errors that can occur when parsing a rarity from string
#[derive(Debug, Error)]
pub enum RarityError {
    /// Error parsing integer value
    #[error(transparent)]
    Parse(#[from] ParseIntError),
    /// Error converting value
    #[error(transparent)]
    FromPrimitive(#[from] TryFromPrimitiveError<Rarity>),
}

impl FromStr for Rarity {
    type Err = RarityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        let value: Rarity = Rarity::try_from_primitive(value)?;
        Ok(value)
    }
}

impl Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
    }
}

#[derive(Debug, Clone)]
pub enum Category {
    Base(BaseCategory),
    Sub(SubCategory),
    Empty,
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Base(left), Self::Base(right)) => left == right,
            (Self::Base(left), Self::Sub(right)) => left == &right.0,
            (Self::Sub(left), Self::Base(right)) => &left.0 == right,
            (Self::Sub(left), Self::Sub(right)) => left == right,
            (Self::Empty, Self::Empty) => true,
            _ => false,
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Base(value) => Display::fmt(value, f),
            Category::Sub(value) => Display::fmt(value, f),
            Category::Empty => Ok(()),
        }
    }
}

#[derive(Debug, Error)]
pub enum CategoryError {
    #[error(transparent)]
    BaseCategory(#[from] BaseCategoryError),
}

impl FromStr for Category {
    type Err = CategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Category::Empty);
        }

        let (base, sub) = s
            .split_once(':')
            .map(|(left, right)| (left, Some(right)))
            .unwrap_or((s, None));

        let base: BaseCategory = base.parse()?;

        Ok(if let Some(sub) = sub {
            Self::Sub(SubCategory(base, Cow::Owned(sub.to_string())))
        } else {
            Self::Base(base)
        })
    }
}

/// Categories of items
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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

#[derive(Debug, Error)]
pub enum BaseCategoryError {
    #[error(transparent)]
    FromPrimitive(#[from] TryFromPrimitiveError<BaseCategory>),

    #[error(transparent)]
    Parse(#[from] ParseIntError),
}

impl FromStr for BaseCategory {
    type Err = BaseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        let value: BaseCategory = BaseCategory::try_from_primitive(value)?;
        Ok(value)
    }
}

impl Display for BaseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
    }
}

/// Sub category within a [BaseCategory]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubCategory(BaseCategory, Cow<'static, str>);

impl Display for SubCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)?;
        f.write_char(':')?;
        Display::fmt(&self.1, f)
    }
}

impl SubCategory {
    // Empty string denotes any sub category
    pub const ALL_CATEGORY: &'static str = "";

    // Weapon categories
    pub const ASSAULT_RIFLE_CATEGORY: &'static str = "AssaultRifle";
    pub const PISTOL_CATEGORY: &'static str = "Pistol";
    pub const SHOTGUN_CATEGORY: &'static str = "Shotgun";
    pub const SNIPER_RIFLE_CATEGORY: &'static str = "SniperRifle";

    // Base weapon types
    pub const ASSAULT_RIFLE: Self = Self::new(BaseCategory::Weapons, Self::ASSAULT_RIFLE_CATEGORY);
    pub const PISTOL: Self = Self::new(BaseCategory::Weapons, Self::PISTOL_CATEGORY);
    pub const SHOTGUN: Self = Self::new(BaseCategory::Weapons, Self::SHOTGUN_CATEGORY);
    pub const SNIPER_RIFLE: Self = Self::new(BaseCategory::Weapons, Self::SNIPER_RIFLE_CATEGORY);

    // Weapon mods
    pub const ASSAULT_RIFLE_MODS: Self =
        Self::new(BaseCategory::WeaponMods, Self::ASSAULT_RIFLE_CATEGORY);
    pub const PISTOL_MODS: Self = Self::new(BaseCategory::WeaponMods, Self::PISTOL_CATEGORY);
    pub const SHOTGUN_MODS: Self = Self::new(BaseCategory::WeaponMods, Self::SHOTGUN_CATEGORY);
    pub const SNIPER_RIFLE_MODS: Self =
        Self::new(BaseCategory::WeaponMods, Self::SNIPER_RIFLE_CATEGORY);

    // Specialized weapons
    pub const ASSAULT_RIFLE_SPECIALIZED: Self = Self::new(
        BaseCategory::WeaponsSpecialized,
        Self::ASSAULT_RIFLE_CATEGORY,
    );
    pub const PISTOL_SPECIALIZED: Self =
        Self::new(BaseCategory::WeaponsSpecialized, Self::PISTOL_CATEGORY);
    pub const SHOTGUN_SPECIALIZED: Self =
        Self::new(BaseCategory::WeaponsSpecialized, Self::SHOTGUN_CATEGORY);
    pub const SNIPER_RIFLE_SPECIALIZED: Self = Self::new(
        BaseCategory::WeaponsSpecialized,
        Self::SNIPER_RIFLE_CATEGORY,
    );

    // Enhanced weapon mod variants
    pub const ASSAULT_RIFLE_MODS_ENHANCED: Self = Self::new(
        BaseCategory::WeaponModsEnhanced,
        Self::ASSAULT_RIFLE_CATEGORY,
    );
    pub const PISTOL_MODS_ENHANCED: Self =
        Self::new(BaseCategory::WeaponModsEnhanced, Self::PISTOL_CATEGORY);
    pub const SHOTGUN_MODS_ENHANCED: Self =
        Self::new(BaseCategory::WeaponModsEnhanced, Self::SHOTGUN_CATEGORY);
    pub const SNIPER_RIFLE_MODS_ENHANCED: Self = Self::new(
        BaseCategory::WeaponModsEnhanced,
        Self::SNIPER_RIFLE_CATEGORY,
    );

    const fn new(base: BaseCategory, value: &'static str) -> Self {
        Self(base, Cow::Borrowed(value))
    }

    pub const fn all(category: BaseCategory) -> Self {
        Self::new(category, Self::ALL_CATEGORY)
    }
}

/// Type used for the weight of a filter result
type FilterWeight = u32;

/// Item filtering
#[derive(Debug, Clone)]
pub enum ItemFilter {
    /// Specific item referenced by [ItemName]
    Named(ItemName),
    /// Require the item to be a specific rarity
    Rarity(Rarity),
    /// Item from a selection of a category
    Category(Category),
    /// Filter based on matching item attributes
    Attributes(HashMap<String, Value>),

    /// Filter matching any of the provided filters
    Any(Vec<ItemFilter>),
    /// Filter matching only when both filters match
    And(Box<ItemFilter>, Box<ItemFilter>),
    /// Filter matching when either filter matches
    Or(Box<ItemFilter>, Box<ItemFilter>),
    /// Filter requiring the other filter does not match
    Not(Box<ItemFilter>),

    /// Filter with an additional weighted randomness amount
    Weighted(Box<ItemFilter>, FilterWeight),
}

impl ItemFilter {
    /// Creates a filter that matches all the provided `rarities`
    pub fn rarities(rarities: &[Rarity]) -> Self {
        Self::Any(rarities.iter().map(|value| Self::Rarity(*value)).collect())
    }

    /// Creates a filter that matches all the provided `categories`
    pub fn categories(categories: &[Category]) -> Self {
        Self::Any(
            categories
                .iter()
                .map(|value| Self::Category(value.clone()))
                .collect(),
        )
    }

    /// Creates an attributes filter from an iterator of key
    /// value pairs
    pub fn attributes<I, K, V>(attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        Self::Attributes(
            attributes
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }

    /// Applies the filter against the provided `item` definition
    /// returns [None] if the value did not match otherwise returns
    /// [Some] with the calculated [FilterWeight]
    pub fn apply_filter(&self, item: &ItemDefinition) -> Option<FilterWeight> {
        match self {
            ItemFilter::Named(name) => {
                if name != &item.name {
                    return None;
                }

                Some(0)
            }
            ItemFilter::Rarity(rarity) => {
                let item_rarity = item.rarity.as_ref()?;
                if rarity != item_rarity {
                    return None;
                }

                Some(0)
            }
            ItemFilter::Category(category) => {
                let item_category = &item.category;

                if item_category != category {
                    return None;
                }

                Some(0)
            }
            ItemFilter::Attributes(_) => todo!(),
            ItemFilter::Any(_) => todo!(),
            ItemFilter::And(_, _) => todo!(),
            ItemFilter::Or(_, _) => todo!(),
            ItemFilter::Not(_) => todo!(),
            ItemFilter::Weighted(_, _) => todo!(),
        }
    }
}
