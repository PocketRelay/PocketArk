use super::HttpError;
use crate::{
    database::entity::{
        currency::CurrencyType, strike_team_mission::StrikeTeamMissionId,
        strike_team_mission_progress::UserMissionState, strike_teams::StrikeTeamId, Currency,
        StrikeTeam, StrikeTeamMission,
    },
    definitions::strike_teams::{StrikeTeamName, StrikeTeamTrait},
    services::activity::ActivityResult,
};
use hyper::StatusCode;
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StrikeTeamError {
    #[error("Team on mission")]
    TeamOnMission,
    #[error("Strike team doesn't exist")]
    UnknownTeam,
    #[error("Strike team mission doesn't exist")]
    UnknownMission,
    #[error("Unknown equipment item")]
    UnknownEquipmentItem,
    /// Cannot recruit any more teams
    #[error("Maximum number of strike teams reached")]
    MaxTeams,
}

impl HttpError for StrikeTeamError {
    fn status(&self) -> StatusCode {
        match self {
            StrikeTeamError::MaxTeams | StrikeTeamError::TeamOnMission => StatusCode::CONFLICT,
            StrikeTeamError::UnknownTeam
            | StrikeTeamError::UnknownEquipmentItem
            | StrikeTeamError::UnknownMission => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveMissionResponse {
    pub team: StrikeTeamWithMission,
    pub mission_successful: bool,
    pub traits_acquired: Vec<StrikeTeamTrait>,
    pub activity_response: ActivityResult,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    pub currency_balance: Currency,
    pub team: StrikeTeam,
    pub next_purchase_cost: Option<u32>,
}
#[derive(Debug, Deserialize)]
pub struct PurchaseQuery {
    pub currency: CurrencyType,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamsResponse {
    pub teams: StrikeTeamsList,
    pub min_specialization_level: u32,
    pub next_purchase_costs: HashMap<CurrencyType, u32>,
    pub inventory_item_limit: usize,
    pub inventory_item_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamsList {
    pub total_count: usize,
    pub list: Vec<StrikeTeamWithMission>,
    pub cap: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamWithMission {
    #[serde(flatten)]
    pub team: StrikeTeam,
    pub mission: Option<StrikeTeamActiveMission>,
}

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamActiveMission {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub name: StrikeTeamMissionId,
    pub live_mission: StrikeTeamMissionWithState,
    pub finish_time: Option<DateTimeUtc>,
    pub successful: bool,
    pub earn_negative_trait: bool,
}

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamMissionSpecific {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub name: StrikeTeamMissionId,
    pub live_mission: StrikeTeamMissionWithState,

    pub finish_time: DateTimeUtc,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamMissionWithState {
    #[serde(flatten)]
    pub mission: StrikeTeamMission,

    pub user_mission_state: UserMissionState,
    pub seen: bool,
    pub completed: bool,
}

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamSuccessRate {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub id: StrikeTeamId,
    pub name: StrikeTeamName,
    #[serde_as(as = "serde_with::Map<serde_with::DisplayFromStr, _>")]
    pub mission_success_rate: Vec<(StrikeTeamMissionId, f32)>,
}
