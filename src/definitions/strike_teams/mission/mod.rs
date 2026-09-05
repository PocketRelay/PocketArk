use crate::{
    database::entity::strike_team_mission::MissionAccessibility,
    definitions::{
        i18n::{I18n, I18nDesc, I18nName, Localized},
        shared::CustomAttributes,
        strike_teams::{
            StrikeTeams,
            mission::{
                level::MissionLevel, mission_type::MissionType, modifier::MissionModifier,
                rewards::MissionRewards, tag::MissionTag, wave::MissionWave,
            },
        },
    },
};
use anyhow::Context;
use chrono::Utc;
use rand::{Rng, seq::SliceRandom};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use strum::Display;
use uuid::Uuid;

pub mod level;
pub mod mission_type;
pub mod modifier;
pub mod rewards;
pub mod tag;
pub mod wave;

const STRIKE_TEAM_MISSION_DEFINITIONS: &str =
    include_str!("../../../resources/data/strikeTeamMissions.json");

/// Collection of mission definitions
#[derive(Deserialize)]
pub struct MissionDefinitions {
    /// Collection of missions for each difficulty level
    pub difficulty: HashMap<MissionDifficulty, MissionTypeGroup>,
    /// Collection of special missions that aren't given by random
    pub special: Vec<MissionDefinition>,
}

impl MissionDefinitions {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        let this: MissionDefinitions = serde_json::from_str(s)?;
        Ok(this)
    }

    pub fn load() -> serde_json::Result<Self> {
        Self::from_str(STRIKE_TEAM_MISSION_DEFINITIONS)
    }

    pub fn by_difficulty(
        &self,
        difficulty: MissionDifficulty,
        apex: bool,
    ) -> anyhow::Result<&[MissionDefinition]> {
        let difficulty_group = self
            .difficulty
            .get(&difficulty)
            .context("Missing difficulty group")?;

        Ok(match apex {
            true => &difficulty_group.apex,
            false => &difficulty_group.standard,
        })
    }

    pub fn random_by_difficulty<R>(
        &self,
        rng: &mut R,
        difficulty: MissionDifficulty,
        apex: bool,
    ) -> anyhow::Result<&MissionDefinition>
    where
        R: Rng,
    {
        let missions = self.by_difficulty(difficulty, apex)?;
        let mission = missions.choose(rng).context("Failed to choose mission")?;
        Ok(mission)
    }
}

impl Localized for MissionDefinitions {
    fn localize(&mut self, i18n: &I18n) {
        self.difficulty
            .iter_mut()
            // Iterate all difficulty based missions
            .flat_map(|(_, group)| group.standard.iter_mut().chain(group.apex.iter_mut()))
            // Include special missions
            .chain(self.special.iter_mut())
            .for_each(|definition| definition.localize(i18n))
    }
}

/// Mission definitions grouped based on the
/// different types (standard and apex)
#[derive(Deserialize)]
pub struct MissionTypeGroup {
    pub standard: Vec<MissionDefinition>,
    pub apex: Vec<MissionDefinition>,
}

/// Definition for a mission
#[derive(Deserialize)]
pub struct MissionDefinition {
    /// The mission descriptor
    pub descriptor: MissionDescriptor,
    /// The mission accessibility
    pub accessibility: MissionAccessibility,
    /// Optional collection of waves for custom missions
    #[serde(default)]
    pub waves: Option<Vec<MissionWave>>,
    /// Optional overridden mission rewards
    #[serde(default)]
    pub rewards: Option<MissionRewards>,
}

impl Localized for MissionDefinition {
    fn localize(&mut self, i18n: &I18n) {
        self.descriptor.localize(i18n);
    }
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MissionDifficulty {
    #[strum(serialize = "bronze")]
    Bronze,
    #[strum(serialize = "silver")]
    Silver,
    #[strum(serialize = "gold")]
    Gold,
    #[strum(serialize = "platinum")]
    Platinum,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct MissionDescriptor {
    /// Unique ID for the mission descriptor
    pub name: Uuid,

    /// Attributes for the mission descriptor
    /// contains the icons for the descriptor
    #[serde(default)]
    pub custom_attributes: CustomAttributes,

    /// Localized name for the mission type
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized description for the mission type
    #[serde(flatten)]
    pub i18n_desc: Option<I18nDesc>,
}

impl Localized for MissionDescriptor {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        if let Some(i18n_desc) = &mut self.i18n_desc {
            i18n_desc.localize(i18n);
        }
    }
}

/// Data required for building and creating a new
/// strike team mission
/// (Passed to the database layer)
pub struct StrikeTeamMissionData {
    pub descriptor: MissionDescriptor,
    pub mission_type: MissionType,
    pub tags: Vec<MissionTag>,
    pub accessibility: MissionAccessibility,
    pub static_modifiers: Vec<MissionModifier>,
    pub dynamic_modifiers: Vec<MissionModifier>,
    pub rewards: MissionRewards,
    pub custom_attributes: CustomAttributes,
    pub waves: Vec<MissionWave>,
    pub start_seconds: i64,
    pub end_seconds: i64,
    pub sp_length_seconds: u16,
}

/// Generates a random mission for the provided `difficulty` and whether
/// the mission should be an Apex mission
pub fn random_mission<R>(
    rng: &mut R,
    difficulty: MissionDifficulty,
    apex: bool,
) -> anyhow::Result<StrikeTeamMissionData>
where
    R: Rng,
{
    let strike_teams = StrikeTeams::get();

    let accessibility = match (&difficulty, apex) {
        // Platinum can only be played in multiplayer
        (MissionDifficulty::Platinum, _) => MissionAccessibility::MultiPlayer,
        // Apex missions can be either multiplayer or strike team
        (_, true) => MissionAccessibility::Any,
        // Strike team only mission
        (_, false) => MissionAccessibility::SinglePlayer,
    };

    let mission = strike_teams
        .missions
        .random_by_difficulty(rng, difficulty, apex)?;
    let descriptor = mission.descriptor.clone();

    // Get the default mission type
    let mission_type = MissionType::default();
    let level = MissionLevel::random(rng)?;

    let enemy_tag = strike_teams.tags.random_enemy(rng)?;
    let mission_tags = strike_teams.tags.random_missions(rng, 2);

    // Create the collection of tags
    let tags: Vec<MissionTag> = std::iter::once(enemy_tag)
        .chain(mission_tags)
        .cloned()
        .collect();

    // Create the modifiers
    let static_modifiers: Vec<MissionModifier> =
        MissionModifier::static_modifiers(difficulty, enemy_tag, level);
    let dynamic_modifiers: Vec<MissionModifier> = MissionModifier::dynamic_modifiers(rng)?;

    // Create the mission rewards
    let rewards = mission
        .rewards
        .clone()
        .unwrap_or_else(|| MissionRewards::new(difficulty, mission.accessibility));
    let custom_attributes = CustomAttributes::default();

    // Get the custom wave definitions or empty list
    let waves = mission.waves.clone().unwrap_or_default();

    let now = Utc::now().timestamp();

    // Mission starts immediately and ends after 24 hours
    let start_seconds = now;
    let end_seconds = now + 86400 /* 24 hours */;

    let mut sp_length_seconds = rng.gen_range(3000..=9000);
    // Apex missions have an additional duration added
    if apex {
        sp_length_seconds += rng.gen_range(1000..=3000);
    }

    Ok(StrikeTeamMissionData {
        descriptor,
        mission_type,
        accessibility,
        tags,
        static_modifiers,
        dynamic_modifiers,
        rewards,
        custom_attributes,
        waves,
        start_seconds,
        end_seconds,
        sp_length_seconds,
    })
}
