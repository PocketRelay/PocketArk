use std::future::Future;

use crate::database::DbResult;
use crate::definitions::shared::CustomAttributes;
use crate::definitions::strike_teams::{
    MissionDescriptor, MissionModifier, MissionRewards, MissionType, MissionWave,
};
use crate::definitions::strike_teams::{MissionTag, StrikeTeamMissionData};
use sea_orm::{ActiveValue::Set, prelude::*};
use sea_orm::{InsertResult, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

use super::strike_team_mission_progress::UserMissionState;
use super::{SeaJson, StrikeTeamMissionProgress, User};

/// Strike team mission ID keying has been replaced with integer keys rather than the UUIDs
/// used by the official game, this is because its *very* annoying to work with
/// UUIDs as primary keys in the SQLite database (Basically defeats the purpose of SeaORM)
pub type StrikeTeamMissionId = u32;

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "strike_team_missions")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Unique ID of the strike team mission
    #[sea_orm(primary_key)]
    #[serde(rename = "name")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub id: StrikeTeamMissionId,
    /// The mission descriptor details
    pub descriptor: MissionDescriptor,
    /// The mission type details
    pub mission_type: MissionType,
    /// Mission accessibility
    pub accessibility: MissionAccessibility,
    /// Custom defined mission waves
    pub waves: SeaJson<Vec<MissionWave>>,
    /// Mission tags
    pub tags: SeaJson<Vec<MissionTag>>,
    /// Static mission modifiers
    pub static_modifiers: SeaJson<Vec<MissionModifier>>,
    /// Dynamic mission modifiers
    pub dynamic_modifiers: SeaJson<Vec<MissionModifier>>,
    /// The mission rewards
    pub rewards: MissionRewards,
    /// Custom attributes associated with the mission
    pub custom_attributes: CustomAttributes,
    /// The time in seconds when the mission became available
    pub start_seconds: i64,
    /// The time in seconds when the mission is no longer available
    pub end_seconds: i64,
    /// The time in seconds the mission will take to complete (Strike teams)
    pub sp_length_seconds: u16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::strike_team_mission_progress::Entity")]
    MissionProgress,
}

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
    pub fn by_id<C>(
        db: &C,
        id: StrikeTeamMissionId,
    ) -> impl Future<Output = DbResult<Option<Self>>> + Send + '_
    where
        C: ConnectionTrait + Send,
    {
        Entity::find_by_id(id).one(db)
    }

    /// Gets all missions that are still available
    ///
    /// TODO: Also need to check progress tables for all user specific missions
    /// that are still awaiting completion
    pub async fn visible_missions<C>(
        db: &C,
        user: &User,
        current_time: i64,
    ) -> DbResult<Vec<(Self, Option<StrikeTeamMissionProgress>)>>
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .find_also_related(super::strike_team_mission_progress::Entity)
            .filter(
                Column::EndSeconds.gt(current_time).or(
                    super::strike_team_mission_progress::Column::UserMissionState
                        .is_in([
                            UserMissionState::PendingResolve,
                            UserMissionState::InProgress,
                        ])
                        .and(super::strike_team_mission_progress::Column::UserId.eq(user.id)),
                ),
            )
            .all(db)
            .await
    }

    /// Gets all missions that are still available
    ///
    /// TODO: Also need to check progress tables for all user specific missions
    /// that are still awaiting completion
    pub async fn available_missions<C>(
        db: &C,
        user: &User,
        current_time: i64,
    ) -> DbResult<Vec<(Self, Option<StrikeTeamMissionProgress>)>>
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .find_also_related(super::strike_team_mission_progress::Entity)
            .filter(
                Column::EndSeconds.gt(current_time).and(
                    super::strike_team_mission_progress::Column::UserMissionState
                        .is_null()
                        .or(
                            super::strike_team_mission_progress::Column::UserMissionState
                                .eq(UserMissionState::Available)
                                .and(
                                    super::strike_team_mission_progress::Column::UserId.eq(user.id),
                                ),
                        ),
                ),
            )
            .all(db)
            .await
    }

    /// Finds the newest strike team mission
    pub fn newest_mission<C>(db: &C) -> impl Future<Output = DbResult<Option<i64>>> + '_
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .select_only()
            // Select only the start seconds
            .column(Column::StartSeconds)
            // Order by the newest
            .order_by_desc(Column::StartSeconds)
            .into_tuple::<i64>()
            .one(db)
    }

    #[allow(unused)]
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

impl Related<super::strike_team_mission_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MissionProgress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
