use crate::{
    database::entity::currency::CurrencyType, http::models::mission::MissionActivity,
    utils::models::DateDuration,
};

use super::{items::ItemName, match_data::ActivityDescriptor};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::{collections::HashSet, process::exit};
use uuid::Uuid;

pub const CHALLENGE_DEFINITIONS: &str =
    include_str!("../../resources/data/challengeDefinitions.json");

pub struct ChallengesService {
    pub defs: Vec<ChallengeDefinition>,
}

#[test]
fn test() {
    let defs: Vec<ChallengeDefinition> = serde_json::from_str(CHALLENGE_DEFINITIONS).unwrap();
    let mut values = HashSet::new();
    for def in defs {
        for category in def.categories {
            values.insert(category);
        }
    }
    println!("{:?}", values);
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

/// Type alias for a [Uuid] representing the name of a [ChallengeDefinition]
pub type ChallengeName = Uuid;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeDefinition {
    /// Unique name for the challenge (UUID)
    pub name: ChallengeName,
    /// Unused by the game and always left empty
    pub description: String,
    /// Likely added to disable a challenge so that it can't be gained
    pub enabled: bool,
    /// List of categories for grouping the challenge ("0", "1", "2", "4")
    pub categories: Vec<String>,
    /// Whether the challenge can be repeated
    pub can_repeat: bool,
    /// Whether the challenge is limited time
    pub limited_availability: bool,

    pub i18n_title: String,
    pub i18n_incomplete: String,
    pub i18n_complete: String,
    pub i18n_notification: String,
    pub i18n_multi_player_notification: String,
    pub i18n_reward_description: String,
    pub i18n_description: Option<String>,

    pub loc_title: String,
    pub loc_description: Option<String>,

    /// Number of challenge points to award
    /// TODO: This needs to be handled
    pub point_value: Option<u32>,

    /// Counters are stored as an array *however* from all of the challenges defined in
    /// the based game they *always* only have one counter.
    pub counters: Vec<ChallengeCounter>,

    /// Extra custom attributes. Mostly related to textures, conditional hiding, and display order
    pub custom_attributes: Map<String, Value>,

    /// Duration for which the challenge will be available
    pub available_duration: DateDuration,
    /// Duration for which the challenge will be visible
    pub visible_duration: DateDuration,

    /// Collection of challenges that parent this challenge
    pub parents: Vec<ChallengeName>,

    /// TODO: Giving challenge rewards is not yet implemented
    pub reward: ChallengeReward,

    /// Unknown usage. Possibly for shared player-base wide challenges..?
    pub community: bool,
}

impl ChallengeDefinition {
    /// Attempts to find
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
    /// Finds an [ActivityDescriptor] from this counters collection of [ChallengeCounter::activities]
    /// that matches the provided mission `activity`
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

/// Represents all the rewards that should be given for a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeReward {
    /// Currency rewards
    pub currencies: Vec<CurrencyReward>,
    /// XP rewards
    pub xp: Vec<Value>,
    /// Item rewards
    pub items: Vec<ItemReward>,
    /// Entitlement rewards
    pub entitlements: Vec<Value>,
}

/// Representing a type of currency to be given as a reward
/// for completing a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyReward {
    /// The type of currency
    pub name: CurrencyType,
    /// The amount of the currency
    pub value: u32,
}

/// Represents an item which should be given as a reward for
/// completing a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReward {
    /// The [ItemName] of the item to give
    pub name: ItemName,
    /// How much of the item to give
    pub count: u32,
    /// The namespace to store the item under
    pub namespace: String,
}
