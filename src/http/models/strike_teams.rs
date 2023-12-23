use super::HttpError;
use crate::{
    database::entity::{currency::CurrencyType, Currency, StrikeTeam},
    definitions::striketeams::{StrikeTeamWithMission, TeamTrait},
    services::activity::ActivityResult,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StrikeTeamError {
    /// Article cannot be purchased with the requested currency
    #[error("Invalid currency")]
    InvalidCurrency,
    /// User doesn't have enough currency to purchase the item
    #[error("Currency balance cannot be less than 0.")]
    InsufficientCurrency,
    #[error("Strike team doesn't exist")]
    UnknownTeam,
    #[error("Unknown equipment item")]
    UnknownEquipmentItem,
    /// Cannot recruit any more teams
    #[error("Maximum number of strike teams reached")]
    MaxTeams,
}

impl HttpError for StrikeTeamError {
    fn status(&self) -> StatusCode {
        match self {
            StrikeTeamError::InvalidCurrency => StatusCode::BAD_REQUEST,
            StrikeTeamError::InsufficientCurrency | StrikeTeamError::MaxTeams => {
                StatusCode::CONFLICT
            }
            StrikeTeamError::UnknownTeam | StrikeTeamError::UnknownEquipmentItem => {
                StatusCode::NOT_FOUND
            }
        }
    }
}

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
