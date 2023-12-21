//! Skill structures and logic

use anyhow::Context;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::str::FromStr;
use uuid::Uuid;

use crate::utils::models::{LocaleName, LocaleNameWithDesc};

/// Skill definitions (64)
const SKILL_DEFINITIONS: &str = include_str!("../../resources/data/skillDefinitions.json");

/// Type alias for a [Uuid] that represents a [SkillDefinition] name
pub type SkillDefinitionName = Uuid;

/// Type alias for a [String] that represents a [Skill] name
pub type SkillName = String;

/// Collection of skill definitions
pub struct SkillDefinitions {
    /// The collection of skill definitions
    pub values: Vec<SkillDefinition>,
}

impl SkillDefinitions {
    /// Creates and loads the skill definitions from [LEVEL_TABLE_DEFINITIONS]
    pub fn new() -> anyhow::Result<Self> {
        let values: Vec<SkillDefinition> =
            serde_json::from_str(SKILL_DEFINITIONS).context("Failed to parse skill definitions")?;

        debug!("Loaded {} skill definition(s)", values.len());

        Ok(Self { values })
    }

    /// Find a [SkillDefinition] by its `name`
    pub fn get(&self, name: &SkillDefinitionName) -> Option<&SkillDefinition> {
        self.values
            .iter()
            .find(|definition| definition.name.eq(name))
    }
}

/// Represents a skill/ability that a character can have
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillDefinition {
    /// Unique identifier for this skill definition
    pub name: SkillDefinitionName,
    /// The different tiers of the skill definition
    pub tiers: Vec<SkillDefinitionTier>,
    /// Additional attributes associated with the skill, appears to be
    /// used for things like the skill icons, active power, and groupings
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
    /// Timestamp for when the skill definition was created
    pub timestamp: DateTime<Utc>,
    /// Localized name and description of the skill definition
    #[serde(flatten)]
    pub local: LocaleNameWithDesc,
}

/// Tier of a [Skill]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillDefinitionTier {
    /// The tier index for this tier
    pub tier: u8,
    /// Additional custom attributes for the tier, appears unused
    /// by the default game definitions
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
    /// Conditions required for unlocking this tier
    pub unlock_conditions: Vec<UnlockCondition>,
    /// Skills this tier includes
    pub skills: Vec<Skill>,
}

/// Unlock condition requirements
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnlockCondition {
    /// The collection of conditions for unlocking
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub conditions: Vec<(Condition, ConditionInt)>,
}

/// Conditional string, needs to be parsed at some point
type Condition = String;
/// Condition value either 1 or 0
type ConditionInt = u8;

/// Defines a skill
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Skill {
    /// The internal game name for the skill
    ///
    /// (e.g game/progression/singleplayer/skills/combat/skill_flakcannon4_powercells)
    pub name: SkillName,

    /// Custom attributes associated with the skill
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
    /// Conditions required for unlocking this skill
    pub unlock_conditions: Vec<UnlockCondition>,
    /// Different levels of the skill
    pub levels: Vec<SkillLevel>,

    /// Localized name and description of the skill
    #[serde(flatten)]
    pub local: LocaleNameWithDesc,
}

/// Defines a level of a skill
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillLevel {
    /// The level number for this skill level
    pub level: u8,
    /// Additional custom attributes for the level, appears unused
    /// by the default game definitions
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
    /// Conditions required for unlocking this level
    pub unlock_conditions: Vec<UnlockCondition>,
    /// The cost of the level in skill points
    pub cost: SkillLevelCost,
    /// Additional bonus attributes, appears unused by the game
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub bonus: Vec<(String, serde_json::Value)>,
}

/// Defines a reference to some value seperated by colons
pub struct PathRef(Vec<String>);

impl FromStr for PathRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(':').map(|value| value.to_string()).collect();
        Ok(Self(parts))
    }
}

/// Defines the cost for unlocking a [SkillLevel]
///
/// This should've probabbly been dynamic however with the strict
/// typing this would likely be difficult
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillLevelCost {
    /// The skill points cost
    #[serde(rename = "character:points:MEA_skill_points")]
    pub skill_points: u32,
}

#[cfg(test)]
mod test {
    use super::{SkillDefinition, SKILL_DEFINITIONS};

    /// Tests ensuring the skill definitions can be parsed
    /// correctly from the resource file
    #[test]
    fn ensure_parsing_succeed() {
        let _: Vec<SkillDefinition> = serde_json::from_str(SKILL_DEFINITIONS).unwrap();
    }
}
