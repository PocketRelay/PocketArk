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
//! https://masseffectandromeda.fandom.com/wiki/Character_kit

use crate::definitions::{
    i18n::{I18nDescription, I18nName},
    items::{InventoryNamespace, ItemLink, ItemName},
    level_tables::LevelTableName,
    shared::CustomAttributes,
    skills::SkillTree,
};
use anyhow::Context;
use log::debug;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::HashMap, sync::OnceLock};
use uuid::Uuid;

/// Class definitions (36)
const CLASS_DEFINITIONS: &str = include_str!("../resources/data/characterClasses.json");

/// Collection of class definitions and lookup tables for finding
/// classes based on certain fields
pub struct Classes {
    /// Collection of classes
    pub values: Vec<Class>,
    /// Lookup table for finding a [Class] by its name
    lookup_by_name: HashMap<ClassName, usize>,
    /// Lookup table for finding a [Class] by its associated item name
    lookup_by_item: HashMap<ItemName, usize>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<Classes> = OnceLock::new();

impl Classes {
    /// Gets a static reference to the global [Classes] collection
    pub fn get() -> &'static Classes {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let values: Vec<Class> =
            serde_json::from_str(CLASS_DEFINITIONS).context("Failed to load class definitions")?;

        debug!("Loaded {} class definition(s)", values.len());

        // Generate the lookup maps
        let (lookup_by_name, lookup_by_item) = values
            .iter()
            .enumerate()
            .map(|(index, class)| ((class.name, index), (class.item_link.1, index)))
            .unzip();

        Ok(Self {
            values,
            lookup_by_item,
            lookup_by_name,
        })
    }

    pub fn all(&self) -> &[Class] {
        &self.values
    }

    /// Finds a class definition by its `name`
    #[allow(unused)]
    pub fn by_name(&self, name: &ClassName) -> Option<&Class> {
        self.lookup_by_name
            .get(name)
            .map(|index| &self.values[*index])
    }

    /// Finds a class definition by its associated item
    pub fn by_item(&self, item: &ItemName) -> Option<&Class> {
        self.lookup_by_item
            .get(item)
            .map(|index| &self.values[*index])
    }
}

/// Type alias for a [Uuid] that represents a [Class] name
pub type ClassName = Uuid;

/// Represents a "class" of a character, unlike ME3 the term class in this
/// game doesn't refer to the type like "Adept", "Soldier", etc instead it
/// refers
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

    /// Default collection of points (Appears as always empty)
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub points: Vec<(String, u32)>,

    /// Default attributes for the character
    pub attributes: CharacterAttributes,
    /// Default bonus for the character
    pub bonus: CharacterBonus,
    /// Custom attributes for the class, contains customization options for
    /// the class health, shields, icons, etc
    pub custom_attributes: CustomAttributes,

    /// Default skill tree configuration, this is cloned and stored in the
    /// character data when created
    pub skill_trees: Vec<SkillTree>,

    /// Default equipment for the character
    pub default_equipments: Vec<CharacterEquipment>,

    /// Default customization data
    pub default_customization: CustomizationMap,
    /// Default namespace for the character
    pub inventory_namespace: InventoryNamespace,
    /// Possibly to generate the default inventory namespace by default, however
    /// always false in base game definitions
    pub autogenerate_inventory_namespace: bool,

    /// Unknown usage
    pub initial_active_candidate: bool,

    /// Same as `inventory_namespace`
    pub default_namespace: InventoryNamespace,

    /// Localized character name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized character description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

pub type CharacterBonus = serde_json::Map<String, serde_json::Value>;

/// Game mapping for different kinds of character points,
/// simplified for this implementation to the only kind of
/// point made use of (Skill points)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PointMap {
    /// Skill points in the point map
    #[serde(rename = "MEA_skill_points")]
    pub skill_points: Option<u32>,
}

/// Map of character attributes
///
/// Stored on the server as a [Vec] of tuples because the server never
/// needs to actually read the contents of the map
#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct CharacterAttributes(
    #[serde_as(as = "serde_with::Map<_, _>")] Vec<(String, serde_json::Value)>,
);

/// Map of character customization data
///
/// Stored on the server as a [Vec] of tuples because the server never
/// needs to actually read the contents of the map
#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct CustomizationMap(
    #[serde_as(as = "serde_with::Map<_, _>")] Vec<(String, CustomizationEntry)>,
);

impl CustomizationMap {
    pub fn set(&mut self, key: String, entry: CustomizationEntry) {
        if let Some((_, value)) = self.0.iter_mut().find(|(k, _)| key.eq(k)) {
            *value = entry;
        } else {
            self.0.push((key, entry))
        }
    }
}

/// Customization entry structure for characters, contains the
/// visual customizations of different parameters
#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationEntry {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub value_x: f32,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub value_y: f32,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub value_z: f32,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub value_w: f32,
    #[serde(rename = "type")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub ty: u32,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub param_id: u32,
}

/// Different equipment slot names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum EquipmentSlot {
    WeaponSlot1,
    WeaponSlot2,
    EquipmentSlot,
    EquipmentHistorySlot,
    Booster1,
    Booster2,
    BannerSlot,
}

/// Character equipment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterEquipment {
    /// The slot the equipment is in
    pub slot: EquipmentSlot,
    /// The name of the item in the equipment slot
    pub name: NameOrEmpty,
    /// Items attached to the equipment
    pub attachments: Vec<ItemName>,
}

/// Can be either an item name or empty string,
///
/// [CharacterEquipment::name] may be an empty string if the slot is cleared
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NameOrEmpty {
    #[serde(rename = "")]
    Empty,
    #[serde(untagged)]
    Name(ItemName),
}

#[cfg(test)]
mod test {
    use super::Classes;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = Classes::load().unwrap();
    }
}
