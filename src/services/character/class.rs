//! Character class structures and logic
//!
//!
//! Unlike ME3 the term class in this game doesn't refer to a type of
//! character ("Solider", "Adept", etc).
//!
//! Instead every character has their own "class" which defines information about the
//! character such as the level tables to use, default equipment, default customization,
//! the item associated with the character, etc
//!

use serde::{ser::SerializeMap, Deserialize, Serialize};
use uuid::Uuid;

use crate::{services::items::ItemLink, utils::models::LocaleNameWithDesc};

use super::levels::LevelTableName;

/// Type alias for a [Uuid] that represents a [Class] name
pub type ClassName = Uuid;

/// Represents a "class" of a character, unlike ME3 the term class in this
/// game doesn't refer to the type like "Adept", "Soldier", etc instead it
/// refers
#[derive(Debug, Deserialize)]
pub struct Class {
    /// Unique identifier for this class
    pub name: ClassName,

    /// The name of the level table to use for this characters XP and leveling
    pub level_name: LevelTableName,
    /// The name of the level table to use for this characters prestige XP and
    /// leveling
    pub prestige_level_name: LevelTableName,
    /// Link to the item representing this character
    pub item_link: ItemLink,

    /// Character class name and description with localized version
    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

impl Serialize for Class {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_map(None)?;

        // value.serialize_entry("n", value)
        value.end()
    }
}

/// Defines the
pub struct CharacterSkillTreeEntry {
    pub name: 
}
