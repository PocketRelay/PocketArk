use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FinishMissionRequest {
    pub duration_sec: u64,
    pub percent_complete: u8,
    pub extraction_state: String,
    pub modifiers: Vec<MissionModifier>,
    pub match_id: String,
    pub player_data: Vec<MissionPlayerData>,
    pub version: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionModifier {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MissionPlayerData {
    pub persona_id: u32,
    pub nucleus_id: u32,
    pub score: u32,
    pub modifiers: Vec<Value>,
    pub activity_report: MissionActivityReport,
    pub stats: HashMap<String, Value>,
    pub present_at_end: bool,
    pub waves_complete: u8,
    pub waves_in_match: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionActivityReport {
    pub name: String,
    pub activities: Vec<MissionActivity>,
    pub options: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionActivity {
    pub name: Uuid,
    pub attributes: HashMap<String, Value>,
}
