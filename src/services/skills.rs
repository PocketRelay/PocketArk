//! Skill structures and logic
//!
//! https://masseffectandromeda.fandom.com/wiki/Character_Customization_(multiplayer)#Skills

use anyhow::Context;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{str::FromStr, sync::OnceLock};
use uuid::Uuid;

use super::i18n::{I18nDescription, I18nName};

/// Skill definitions (64)
const SKILL_DEFINITIONS: &str = include_str!("../resources/data/skillDefinitions.json");

/// Type alias for a [Uuid] that represents a [SkillDefinition] name
pub type SkillDefinitionName = Uuid;

/// Type alias for a [String] that represents a [Skill] name
pub type SkillName = String;

/// Collection of skill definitions
pub struct SkillDefinitions {
    /// The collection of skill definitions
    pub values: Vec<SkillDefinition>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<SkillDefinitions> = OnceLock::new();

impl SkillDefinitions {
    /// Gets a static reference to the global [ChallengeDefinitions] collection
    pub fn get() -> &'static SkillDefinitions {
        STORE.get_or_init(|| Self::new().unwrap())
    }

    /// Creates and loads the skill definitions from [LEVEL_TABLE_DEFINITIONS]
    fn new() -> anyhow::Result<Self> {
        let values: Vec<SkillDefinition> =
            serde_json::from_str(SKILL_DEFINITIONS).context("Failed to parse skill definitions")?;

        debug!("Loaded {} skill definition(s)", values.len());

        Ok(Self { values })
    }

    /// Find a [SkillDefinition] by its `name`
    pub fn by_name(&self, name: &SkillDefinitionName) -> Option<&SkillDefinition> {
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
    /// Localized skill name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized skill description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
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

    /// Localized skill name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized skill description
    #[serde(flatten)]
    pub i18n_description: Option<I18nDescription>,
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

/// Represents a skill tree definition
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillTree {
    /// Name of the skill definition this tree is for
    pub name: SkillDefinitionName,
    /// The tree of tiers
    pub tree: Vec<SkillTreeTier>,
    /// Optional timestamp for when the tree was created
    pub timestamp: Option<DateTime<Utc>>,
    /// UNknown usage
    pub obsolete: bool,
}

/// Represents a tier of a skill tree definition
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillTreeTier {
    /// The skill tier index
    pub tier: u8,
    /// Collection of skills and whether they are unlocked or not
    #[serde_as(as = "serde_with::Map<_, serde_with::BoolFromInt>")]
    pub skills: Vec<(SkillName, bool)>,
}

impl SkillTreeTier {
    /// Sets a skill tree tier value
    pub fn set_skill(&mut self, name: SkillName, value: bool) {
        if let Some((_, v)) = self.skills.iter_mut().find(|(k, _)| name.eq(k)) {
            *v = value;
        } else {
            self.skills.push((name, value))
        }
    }
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
