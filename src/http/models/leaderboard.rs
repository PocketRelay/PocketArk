use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::utils::models::LocaleNameWithDesc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardsResponse {
    pub total_count: usize,
    pub list: Vec<LeaderboardCategory>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardCategory {
    pub name: Uuid,

    pub stat_collection_name: Uuid,
    pub stat_owner_name: String,
    pub ranked_stat_name: String,
    pub i18n_ranked_stat: String,
    pub seconds_to_live_after_last_write: u32,
    pub properties: Vec<Value>,
    pub owner_id_type: String,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardRow {
    pub rank: u64,
    pub name: String,
    pub owner_id: u32,
    pub stat_value: f32,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub centered: bool,
    #[serde(default)]
    pub rank_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardIdent {
    pub name: Uuid,
    pub property_value_map: Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub identifier: LeaderboardIdent,
    pub rows: Vec<LeaderboardRow>,
}
