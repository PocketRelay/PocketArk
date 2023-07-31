use crate::{
    database::entity::{Currency, StrikeTeam},
    services::{
        activity::ActivityResult,
        strike_teams::{StrikeTeamEquipment, StrikeTeamWithMission, TeamTrait},
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveMissionResponse {
    pub team: StrikeTeamWithMission,
    pub mission_successful: bool,
    pub traits_acquired: Vec<TeamTrait>,
    pub activity_response: ActivityResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    pub currency_balance: Currency,
    pub team: StrikeTeam,
    pub next_purchase_cost: u32,
}
