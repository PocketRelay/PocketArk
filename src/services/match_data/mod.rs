use std::process::exit;

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

pub const MATCH_BADGE_DEFINITIONS: &str = include_str!("../../resources/data/matchBadges.json");

pub struct MatchDataService {
    pub badges: Vec<Badge>,
}

impl MatchDataService {
    pub fn load() -> Self {
        debug!("Loading match badges");
        let list: Vec<Badge> = match serde_json::from_str(MATCH_BADGE_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load match badge definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} inventory item definition(s)", list.len());
        Self { badges: list }
    }

    pub fn get_by_activity(&self, activity: Uuid) {}
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub name: Uuid,
    pub description: String,
    pub custom_attributes: Map<String, Value>,
    pub enabled: bool,

    pub i18n_title: Option<String>,
    pub i18n_description: Option<String>,
    pub loc_title: Option<String>,
    pub loc_description: Option<String>,

    pub currency: String,

    pub activities: Vec<BadgeActivity>,
    pub levels: Vec<BadgeLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeActivity {
    pub activity_name: Uuid,
    pub filter: Map<String, Value>,
    pub increment_progress_by: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeLevel {
    pub name: String,
    pub img_path: Option<String>,
    #[serde(rename = "imgURLFull")]
    pub img_url_full: Option<String>,
    pub target_count: u32,
    pub xp_reward: u32,
    pub currency_reward: u32,
    pub rewards: Vec<Value>,
    pub custom_attributes: Map<String, Value>,
}
