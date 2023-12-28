use std::future::Future;

use crate::database::DbResult;
use crate::definitions::shared::CustomAttributes;
use crate::definitions::strike_teams::{
    MissionDescriptor, MissionModifier, MissionRewards, MissionType, MissionWave,
};
use crate::definitions::strike_teams::{MissionTag, StrikeTeamMissionData};
use sea_orm::InsertResult;
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::SeaJson;

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
    pub waves: SeaJson<Vec<MissionWave>>,
    /// Mission tags
    pub tags: SeaJson<Vec<MissionTag>>,
    /// Static mission modifiers
    pub static_modifiers: SeaJson<Vec<MissionModifier>>,
    /// Dynamic mission modifiers
    pub dynamic_modifiers: SeaJson<Vec<MissionModifier>>,
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

impl Model {
    /// Finds the newest strike team mission
    pub async fn newest_mission() {
        unimplemented!()
    }

    pub fn create<C>(
        db: &C,
        data: StrikeTeamMissionData,
    ) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        ActiveModel {
            descriptor: Set(data.descriptor),
            mission_type: Set(data.mission_type),
            tags: Set(SeaJson(data.tags)),
            accessibility: Set(data.accessibility),
            static_modifiers: Set(SeaJson(data.static_modifiers)),
            dynamic_modifiers: Set(SeaJson(data.dynamic_modifiers)),
            rewards: Set(data.rewards),
            custom_attributes: Set(data.custom_attributes),
            waves: Set(SeaJson(data.waves)),
            start_seconds: Set(data.start_seconds),
            end_seconds: Set(data.end_seconds),
            sp_length_seconds: Set(data.sp_length_seconds),
            ..Default::default()
        }
        .insert(db)
    }
    pub fn create_many<C>(
        db: &C,
        data: Vec<StrikeTeamMissionData>,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + '_
    where
        C: ConnectionTrait + Send,
    {
        Entity::insert_many(data.into_iter().map(|data| ActiveModel {
            descriptor: Set(data.descriptor),
            mission_type: Set(data.mission_type),
            tags: Set(SeaJson(data.tags)),
            accessibility: Set(data.accessibility),
            static_modifiers: Set(SeaJson(data.static_modifiers)),
            dynamic_modifiers: Set(SeaJson(data.dynamic_modifiers)),
            rewards: Set(data.rewards),
            custom_attributes: Set(data.custom_attributes),
            waves: Set(SeaJson(data.waves)),
            start_seconds: Set(data.start_seconds),
            end_seconds: Set(data.end_seconds),
            sp_length_seconds: Set(data.sp_length_seconds),
            ..Default::default()
        }))
        .exec(db)
    }
}

impl ActiveModelBehavior for ActiveModel {}
