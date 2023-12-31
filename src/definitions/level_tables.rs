//! Leveling table structures and logic

use super::shared::CustomAttributes;
use crate::utils::ImStr;
use anyhow::Context;
use log::debug;
use sea_orm::FromJsonQueryResult;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};
use uuid::Uuid;

/// Level table definitions (7)
const LEVEL_TABLE_DEFINITIONS: &str = include_str!("../resources/data/levelTables.json");

/// Collection of level tables
pub struct LevelTables {
    /// The collection of level tables
    pub values: Vec<LevelTable>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<LevelTables> = OnceLock::new();

impl LevelTables {
    /// Gets a static reference to the global [LevelTables] collection
    pub fn get() -> &'static LevelTables {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    /// Creates and loads the level tables from [LEVEL_TABLE_DEFINITIONS]
    fn load() -> anyhow::Result<Self> {
        let values: Vec<LevelTable> = serde_json::from_str(LEVEL_TABLE_DEFINITIONS)
            .context("Failed to parse level table definitions")?;

        debug!("Loaded {} level table definition(s)", values.len());

        Ok(Self { values })
    }

    /// Find a [LevelTable] by its `name`
    pub fn by_name(&self, name: &LevelTableName) -> Option<&LevelTable> {
        self.values
            .iter()
            .find(|level_table| level_table.name.eq(name))
    }
}

/// Type alias for a [Uuid] representing a [LevelTable] name
pub type LevelTableName = Uuid;

/// Defines a level table which describes how leveling progression
/// should be handled and the associated XP requirements
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTable {
    /// The unique name of this level table
    pub name: LevelTableName,
    /// The collection of table entries
    pub table: Vec<LevelTableEntry>,
    /// Custom additional attributes associated with this table
    pub custom_attributes: CustomAttributes,
}

impl LevelTable {
    /// Computes the new xp and level values from the provided
    /// initial xp, level and the earned xp amount. Uses the
    /// current level table
    pub fn compute_leveling(
        &self,
        mut xp: ProgressionXp,
        mut level: u32,
        xp_earned: u32,
    ) -> (ProgressionXp, u32) {
        xp.current = xp.current.saturating_add(xp_earned);

        // Only continue progression while theres a next level available
        while let Some(next_xp) = self.get_xp_requirement(level + 1) {
            // Don't have enough xp to level up again
            if xp.current < next_xp {
                break;
            }

            // Incrase level
            xp.current -= next_xp;
            level += 1;

            // Update the last and next states
            xp.last = xp.next;
            xp.next = next_xp;
        }

        // Remove any overflow
        xp.current = xp.current.min(xp.next);

        (xp, level)
    }

    /// Gets the XP that is required to reach the provided `level` if the
    /// table contains an entry for it
    pub fn get_xp_requirement(&self, level: u32) -> Option<u32> {
        self.table
            .iter()
            .find(|entry| entry.level == level)
            .map(|entry| entry.xp)
    }

    /// Gets the xp values for the previous, current, and next levels using
    /// the provided `level` as the current level
    pub fn get_xp_values(&self, level: u32) -> Option<(u32, u32, u32)> {
        let current = self.get_xp_requirement(level)?;
        let previous = self
            .get_xp_requirement(level.saturating_sub(1))
            .unwrap_or_default();
        let next = self.get_xp_requirement(level).unwrap_or_default();
        Some((previous, current, next))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTableEntry {
    /// The level this entry is for
    pub level: u32,
    /// The required XP to reach this level entry
    pub xp: u32,
    /// Rewards the level table level provides
    pub rewards: HashMap<ImStr, f64>,
    /// Additional custom attributes (Appears unused by game definitions)
    pub custom_attributes: CustomAttributes,
}

impl Serialize for LevelTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_struct("LevelTable", 7)?;

        value.serialize_field("table", &self.table)?;
        value.serialize_field("name", &self.name)?;

        // Localization data is always empty / null
        value.serialize_field("i18nName", "")?;
        value.serialize_field("i18nDescription", "")?;
        value.serialize_field("locName", &None::<String>)?;
        value.serialize_field("locDescription", &None::<String>)?;

        value.serialize_field("customAttributes", &self.custom_attributes)?;

        value.end()
    }
}

/// Structure for tracking XP progression of a character or strike team
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ProgressionXp {
    /// The previous XP that was reached (last level)
    pub last: u32,
    /// The current XP
    pub current: u32,
    /// The amount of XP for the next level
    pub next: u32,
}

impl From<(u32, u32, u32)> for ProgressionXp {
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            last: value.0,
            current: value.1,
            next: value.2,
        }
    }
}
#[cfg(test)]
mod test {
    use super::LevelTables;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = LevelTables::load().unwrap();
    }
}
