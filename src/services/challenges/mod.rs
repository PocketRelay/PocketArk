use crate::{database::entity::currency::CurrencyType, http::models::mission::MissionActivity};

use super::{items::ItemName, match_data::ActivityDescriptor};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::process::exit;
use uuid::Uuid;

pub const CHALLENGE_DEFINITIONS: &str =
    include_str!("../../resources/data/challengeDefinitions.json");

pub struct ChallengesService {
    pub defs: Vec<ChallengeDefinition>,
}

#[test]
fn test() {
    let defs: Vec<ChallengeDefinition> = serde_json::from_str(CHALLENGE_DEFINITIONS).unwrap();
    for def in defs {
        if def.counters.len() > 1 {
            println!("MANY {} {}", def.name, def.counters.len());
        } else {
            let counter = def.counters.first().unwrap();
            if counter.activities.len() < 1 {
                println!("{}", counter.name);
            }
            // println!("{}", counter.chain_to);
        }
    }
}

impl ChallengesService {
    pub fn new() -> Self {
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

    pub fn get_by_activity(
        &self,
        activity: &MissionActivity,
    ) -> Option<(&ChallengeDefinition, &ChallengeCounter, &ActivityDescriptor)> {
        self.defs
            .iter()
            .find_map(|value| value.get_by_activity(activity))
    }
}

#[derive(Debug, Clone)]
pub struct ChallengeProgressUpdate {
    pub progress: u32,
    pub counter: &'static ChallengeCounter,
    pub definition: &'static ChallengeDefinition,
}

/// Type alias for a [Uuid] representing the name of a [ChallengeDefinition]
pub type ChallengeName = Uuid;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeDefinition {
    pub name: ChallengeName,
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

    /// Counters are stored as an array *however* from all of the challenges defined in
    /// the based game they *always* only have one counter.
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

impl ChallengeDefinition {
    pub fn get_by_activity(
        &self,
        activity: &MissionActivity,
    ) -> Option<(&Self, &ChallengeCounter, &ActivityDescriptor)> {
        self.counters
            .iter()
            .find_map(|counter| counter.get_by_activity(activity))
            .map(|(counter, descriptor)| (self, counter, descriptor))
    }
}

/// Definition for a counter that can be used to track challenge
/// progression
///
/// Contains "i18nTitle" and "i18nDescription" fields however these
/// are both blank and unused
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeCounter {
    /// Name of the counter
    pub name: String,
    /// Possibly used for combining counters...? No usage has been
    /// seen in the actual game defined ones are all blank
    pub chain_to: String,
    /// The value that when reached by [ChallengeCounter::activities] will
    /// count as one completion for the challenge
    pub target_count: u32,
    /// Possibly the interval that after which a challenge counter should be
    /// reset..?
    pub interval: u32,
    /// Collection of [ActivityDescriptor] that can be used for tracking progression
    /// towards this counter.
    ///
    /// Can be empty if activities don't affect this counter
    pub activities: Vec<ActivityDescriptor>,
    /// Usage unknown
    pub aggregate: Option<bool>,
}

impl ChallengeCounter {
    pub fn get_by_activity(
        &self,
        activity: &MissionActivity,
    ) -> Option<(&Self, &ActivityDescriptor)> {
        self.activities
            .iter()
            .find(|value| value.matches(activity))
            .map(|value| (self, value))
    }
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
    pub name: CurrencyType,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReward {
    pub name: ItemName,
    pub count: u32,
    pub namespace: String,
}
