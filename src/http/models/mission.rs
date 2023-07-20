use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{database::entity::Currency, services::challenges::CurrencyReward};

use super::auth::Sku;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteMissionData {
    pub duration_sec: u64,
    pub percent_complete: u8,
    pub extraction_state: String,
    pub modifiers: Vec<MissionModifier>,
    pub match_id: String,
    pub player_data: Vec<MissionPlayerData>,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionModifier {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPlayerData {
    pub persona_id: u32,
    pub nucleus_id: u32,
    pub score: u32,
    pub modifiers: Vec<Value>,
    pub activity_report: MissionActivityReport,
    pub stats: HashMap<String, Value>,
    pub present_at_end: bool,
    pub waves_completed: u8,
    pub waves_in_match: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionActivityReport {
    pub name: String,
    pub activities: Vec<MissionActivity>,
    pub options: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionActivity {
    pub name: String,
    pub attributes: MissionActivityAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionActivityAttributes {
    pub count: u32,
    pub score: u32,

    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartMissionRequest {
    pub modifiers: Vec<MissionModifier>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartMissionResponse {
    pub match_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionDetails {
    pub sku: Sku,
    pub name: String, // is just match ID
    pub duration_sec: u64,
    pub percent_complete: u8,
    pub waves_encountered: u8,
    pub extraction_state: String,
    pub enemy_type: String,
    pub difficulty: String,
    pub map: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub processed: DateTime<Utc>,
    pub player_infos: Vec<MissionPlayerInfo>,
    pub modifiers: Vec<MissionModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPlayerInfo {
    pub activities_processed: bool,
    pub bonuses: Vec<Value>,
    pub activities: Vec<Value>,
    pub badges: Vec<PlayerInfoBadge>,
    pub stats: HashMap<String, Value>,
    pub result: PlayerInfoResult,
    pub pid: u32,
    pub persona_id: u32,
    pub persona_display_name: String,
    pub character_id: Uuid,
    pub character_class: Uuid,
    pub modifiers: Vec<Value>,
    pub session_id: Uuid,
    pub wave_participation: u8,
    pub present_at_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoBadge {
    pub count: u32,
    pub level_name: String,
    pub rewarded_levels: Vec<String>,
    pub name: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoResult {
    pub challenges_updated: HashMap<String, ChallengeUpdate>,
    pub items_earned: Vec<Value>,
    pub xp_earned: u32,
    pub previous_xp: u32,
    pub current_xp: u32,
    pub previous_level: u32,
    pub level: u32,
    pub leveled_up: bool,
    pub score: u32,
    pub total_score: u32,
    pub character_class_name: Uuid,
    pub total_currencies_earned: Vec<CurrencyReward>,
    pub reward_sources: Vec<RewardSource>,
    pub prestige_progression: PrestigeProgression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdate {
    pub challenge_id: Uuid,
    pub counters: Vec<ChallengeUpdateCounter>,
    pub status_change: ChallengeStatusChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeStatusChange {
    Notify,
    Changed,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdateCounter {
    pub name: String,
    pub current_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardSource {
    pub name: String,
    pub xp: u32,
    pub currencies: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeProgression {
    pub before: HashMap<Uuid, PrestigeData>,
    pub after: HashMap<Uuid, PrestigeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeData {
    pub name: Uuid,
    pub level: u32,
    pub xp: u32,
}
