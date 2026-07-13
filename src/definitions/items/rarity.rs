use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, DisplayFromStr, SerializeAs};
use std::{fmt::Display, num::ParseIntError, str::FromStr};
use thiserror::Error;

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

impl Display for ItemRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Enum is formatted as underlying value
        Display::fmt(&(*self as u8), f)
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

impl<'de> Deserialize<'de> for ItemRarity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}

impl Serialize for ItemRarity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DisplayFromStr::serialize_as(self, serializer)
    }
}
