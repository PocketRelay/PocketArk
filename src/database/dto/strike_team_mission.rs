use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::EnumIter;
use uuid::Uuid;

use crate::{
    database::dto::{strike_teams::StrikeTeamId, users::UserId},
    definitions::{
        i18n::Localized,
        shared::CustomAttributes,
        strike_teams::mission::{
            MissionDescriptor, mission_type::MissionType, modifier::MissionModifier,
            rewards::MissionRewards, tag::MissionTag, wave::MissionWave,
        },
    },
};

pub type StrikeTeamMissionId = i64;

/// Enum for the different known mission accessibility types
#[derive(Debug, Clone, EnumIter, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[repr(u8)]
pub enum MissionAccessibility {
    // Strike teams or apex
    Any = 0,
    // Apex only
    #[serde(rename = "Multi_Player")]
    MultiPlayer = 1,
    // Strike teams only
    #[serde(rename = "Single_Player")]
    SinglePlayer = 2,
}

#[derive(Debug, FromRow, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamMissionDto {
    #[serde(skip)]
    pub id: StrikeTeamMissionId,
    /// Internal game name UUID, the game hates us if we try using a string version of our regular ID
    /// and will deadlock the game
    pub name: Uuid,
    /// The mission descriptor details
    #[sqlx(json)]
    pub descriptor: MissionDescriptor,
    /// The mission type details
    #[sqlx(json)]
    pub mission_type: MissionType,
    /// Mission accessibility
    pub accessibility: MissionAccessibility,
    /// Custom defined mission waves
    #[sqlx(json)]
    pub waves: Vec<MissionWave>,
    /// Mission tags
    #[sqlx(json)]
    pub tags: Vec<MissionTag>,
    /// Static mission modifiers
    #[sqlx(json)]
    pub static_modifiers: Vec<MissionModifier>,
    /// Dynamic mission modifiers
    #[sqlx(json)]
    pub dynamic_modifiers: Vec<MissionModifier>,
    /// The mission rewards
    #[sqlx(json)]
    pub rewards: MissionRewards,
    /// Custom attributes associated with the mission
    #[sqlx(json)]
    pub custom_attributes: CustomAttributes,
    /// The time in seconds when the mission became available
    pub start_seconds: i64,
    /// The time in seconds when the mission is no longer available
    pub end_seconds: i64,
    /// The time in seconds the mission will take to complete (Strike teams)
    pub sp_length_seconds: u16,
}

impl Localized for StrikeTeamMissionDto {
    fn localize(&mut self, i18n: &crate::definitions::i18n::I18n) {
        self.descriptor.localize(i18n);
        self.mission_type.localize(i18n);
        self.tags.localize(i18n);
        self.rewards.localize(i18n);
    }
}

#[derive(Debug, FromRow, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamMissionWithProgressDto {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub mission: StrikeTeamMissionDto,

    #[sqlx(flatten)]
    #[serde(flatten)]
    pub progress: StrikeTeamMissionProgressDto,
}

#[derive(Debug, Clone)]
pub struct CreateStrikeTeamMissionDto {
    pub name: Uuid,
    pub descriptor: MissionDescriptor,
    pub mission_type: MissionType,
    pub accessibility: MissionAccessibility,
    pub waves: Vec<MissionWave>,
    pub tags: Vec<MissionTag>,
    pub static_modifiers: Vec<MissionModifier>,
    pub dynamic_modifiers: Vec<MissionModifier>,
    pub rewards: MissionRewards,
    pub custom_attributes: CustomAttributes,
    pub start_seconds: i64,
    pub end_seconds: i64,
    pub sp_length_seconds: u16,
}

#[derive(Debug, FromRow, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct StrikeTeamMissionProgressDto {
    /// The users current mission state
    pub user_mission_state: UserMissionState,
    /// Whether the user has seen the mission
    pub seen: bool,
    /// Whether the mission is completed
    pub completed: bool,
}

/// Enum for the different known currency types
#[derive(
    Debug, Default, EnumIter, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum UserMissionState {
    #[default]
    Available = 0,
    InProgress = 1,
    PendingResolve = 2,
    Completed = 3,
}
