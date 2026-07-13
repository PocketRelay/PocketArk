use anyhow::Context;
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{
    definitions::i18n::{I18n, I18nDesc, I18nName, Localized},
    utils::ImStr,
};

const STRIKE_TEAM_TAG_DEFINITIONS: &str =
    include_str!("../../../resources/data/strikeTeamTags.json");

/// Collection of mission tags, split based on their different types
#[derive(Debug, Serialize, Deserialize)]
pub struct MissionTags {
    /// Mission tags for enemies (To choose which enemy is used)
    pub enemy: Vec<MissionTag>,
    /// Mission specific tags (To choose various factors about the mission i.e night-time)
    pub mission: Vec<MissionTag>,
}

impl MissionTags {
    pub fn from_str(s: &str) -> serde_json::Result<MissionTags> {
        serde_json::from_str(s)
    }

    pub fn load() -> serde_json::Result<Self> {
        Self::from_str(STRIKE_TEAM_TAG_DEFINITIONS)
    }

    pub fn random_enemy<R>(&self, rng: &mut R) -> anyhow::Result<&MissionTag>
    where
        R: Rng,
    {
        self.enemy.choose(rng).context("Failed to choose enemy")
    }

    /// Selects multiple random mission tags
    pub fn random_missions<R>(&self, rng: &mut R, amount: usize) -> Vec<&MissionTag>
    where
        R: Rng,
    {
        self.mission.choose_multiple(rng, amount).collect()
    }
}

impl Localized for MissionTags {
    fn localize(&mut self, i18n: &I18n) {
        self.enemy
            .iter_mut()
            .chain(self.mission.iter_mut())
            .for_each(|value| value.localize(i18n))
    }
}

/// Type alias for a [ImStr] representing a [MissionTag::name]
pub type MissionTagName = ImStr;

/// Represents a tag that a mission can have associated with it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionTag {
    /// Name of the mission tag
    pub name: MissionTagName,
    /// Localized name of the tag
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized description of the tag (Appears unused)
    #[serde(flatten)]
    pub i18n_desc: I18nDesc,
}

impl Localized for MissionTag {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        self.i18n_desc.localize(i18n);
    }
}
