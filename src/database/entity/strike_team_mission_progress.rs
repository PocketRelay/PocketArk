//! Stores the mission progress for an individual user towards a
//! strike team mission

use super::users::UserId;
use super::{strike_team_mission::StrikeTeamMissionId, strike_teams::StrikeTeamId};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "strike_team_mission_progress")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Unique ID of the strike team mission
    #[sea_orm(primary_key)]
    pub mission_id: StrikeTeamMissionId,
    /// The ID of the user this progress is for
    pub user_id: UserId,
    /// The ID of the strike team on the mission
    pub strike_team_id: StrikeTeamId,
    /// The users current mission state
    pub user_mission_state: UserMissionState,
    /// Whether the user has seen the mission
    pub seen: bool,
    /// Whether the mission is completed
    pub completed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

/// Enum for the different known currency types
#[derive(
    Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum UserMissionState {
    Available = 0,
    InProgress = 1,
    PendingResolve = 2,
    Completed = 3,
}

impl Model {}

impl ActiveModelBehavior for ActiveModel {}
