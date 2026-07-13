use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, DisplayFromStr, SerializeAs};
use std::{
    fmt::{Display, Write},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

/// Represents an item category
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Category {
    /// Base category portion
    Base(BaseCategory),
    /// Sub category
    Sub(SubCategory),
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
        DisplayFromStr::serialize_as(self, serializer)
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

impl Display for BaseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
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

/// Sub category within a [BaseCategory]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SubCategory(pub BaseCategory, pub String);

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

impl Display for SubCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)?;
        f.write_char(':')?;
        Display::fmt(&self.1, f)
    }
}

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
