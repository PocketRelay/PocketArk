use std::collections::HashMap;

use crate::{
    database::entity::{Currency, StrikeTeam},
    services::{
        activity::ActivityResult,
        strike_teams::{StrikeTeamWithMission, TeamTrait},
    },
};
use serde::Serialize;
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveMissionResponse {
    pub team: StrikeTeamWithMission,
    pub mission_successful: bool,
    pub traits_acquired: Vec<TeamTrait>,
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

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamsResponse {
    pub teams: StrikeTeamsList,
    pub min_specialization_level: u32,
    pub next_purchase_costs: HashMap<String, u32>,
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
