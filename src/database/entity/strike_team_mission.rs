use crate::definitions::shared::CustomAttributes;
use crate::definitions::strike_teams::MissionTag;
use crate::definitions::strike_teams::{
    MissionDescriptor, MissionModifier, MissionRewards, MissionType, MissionWave,
};
use sea_orm::{prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Strike team mission ID keying has been replaced with integer keys rather than the UUIDs
/// used by the official game, this is because its *very* annoying to work with
/// UUIDs as primary keys in the SQLite database (Basically defeats the purpose of SeaORM)
pub type StrikeTeamMissionId = u32;

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "strike_team_missions")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Unique ID of the strike team mission
    #[sea_orm(primary_key)]
    #[serde(rename = "name")]
    pub id: StrikeTeamMissionId,
    /// The mission descriptor details
    pub descriptor: MissionDescriptor,
    /// The mission type details
    pub mission_type: MissionType,
    /// Mission accessiblity
    pub accessibility: MissionAccessibility,
    /// Custom defined mission waves
    pub waves: MissionWaves,
    /// Mission tags
    pub tags: MissionTags,
    /// Static mission modifiers
    pub static_modifiers: MissionModifiers,
    /// Dynamic mission modifiers
    pub dynamic_modifiers: MissionModifiers,
    /// The mission rewarads
    pub rewards: MissionRewards,
    /// Custom attributes associated with the mission
    pub custom_attributes: CustomAttributes,
    /// The time in seconds when the mission became available
    pub start_seconds: u64,
    /// The time in seconds when the mission is no longer available
    pub end_seconds: u64,
    /// The time in seconds the mission will take to complete (Strike teams)
    pub sp_length_seconds: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct MissionTags(pub Vec<MissionTag>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct MissionWaves(pub Vec<MissionWave>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct MissionModifiers(pub Vec<MissionModifier>);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

/// Enum for the different known currency types
#[derive(
    Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
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

impl Model {}

impl ActiveModelBehavior for ActiveModel {}
