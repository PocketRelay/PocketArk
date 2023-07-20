use std::process::exit;

use crate::database::entity::ChallengeProgress;

use super::match_data::ActivityDescriptor;
use chrono::{DateTime, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

pub const CHALLENGE_DEFINITIONS: &str =
    include_str!("../../resources/data/challengeDefinitions.json");

pub struct ChallengesService {
    pub defs: Vec<ChallengeDefinition>,
}

impl ChallengesService {
    pub fn load() -> Self {
        debug!("Loading challenges");
        let defs: Vec<ChallengeDefinition> = match serde_json::from_str(CHALLENGE_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load challenge definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} challenge definition(s)", defs.len());
        Self { defs }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeDefinition {
    pub name: Uuid,
    pub description: String,
    pub enabled: bool,
    pub categories: Vec<String>,
    pub can_repeat: bool,
    pub limited_availability: bool,
    pub i18n_title: Option<String>,
    pub i18n_incomplete: Option<String>,
    pub i18n_complete: Option<String>,
    pub i18n_notification: Option<String>,
    pub i18n_multi_player_notification: Option<String>,
    pub i18n_reward_description: Option<String>,
    pub point_value: Option<u32>,
    pub counters: Vec<ChallengeCounter>,
    pub custom_attributes: Map<String, Value>,
    pub available_duration: Map<String, Value>,
    pub visible_duration: Map<String, Value>,
    pub parents: Vec<Uuid>,
    pub i18n_description: Option<String>,
    pub reward: ChallengeReward,
    pub community: bool,
    pub loc_title: Option<String>,
    pub loc_description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeCounter {
    pub name: String,
    pub chain_to: String,
    pub target_count: u32,
    pub interval: u32,
    pub i18n_title: Option<String>,
    pub i18n_description: Option<String>,
    pub activities: Vec<ActivityDescriptor>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeReward {
    pub currencies: Vec<CurrencyReward>,
    pub xp: Vec<Value>,
    pub items: Vec<ItemReward>,
    pub entitlements: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyReward {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReward {
    pub name: Uuid,
    pub count: u32,
    pub namespace: String,
}
