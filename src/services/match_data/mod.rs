use crate::{
    database::entity::currency::CurrencyName,
    http::models::mission::{MissionActivity, MissionActivityAttributes},
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, process::exit};
use uuid::Uuid;

pub const MATCH_BADGE_DEFINITIONS: &str = include_str!("../../resources/data/matchBadges.json");
pub const MATCH_MODIFIER_DEFINITIONS: &str =
    include_str!("../../resources/data/matchModifiers.json");

pub struct MatchDataService {
    pub badges: Vec<Badge>,
    pub modifiers: Vec<MatchModifier>,
}

impl MatchDataService {
    pub fn new() -> Self {
        debug!("Loading match badges");
        let badges: Vec<Badge> = match serde_json::from_str(MATCH_BADGE_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load match badge definitions: {}", err);
                exit(1);
            }
        };
        let modifiers: Vec<MatchModifier> = match serde_json::from_str(MATCH_MODIFIER_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load match badge definitions: {}", err);
                exit(1);
            }
        };

        debug!(
            "Loaded {} badges, {} modifier definition(s)",
            badges.len(),
            modifiers.len()
        );
        Self { badges, modifiers }
    }

    pub fn get_by_activity(
        &self,
        activity: &MissionActivity,
    ) -> Option<(&Badge, u32, Vec<&BadgeLevel>)> {
        let (badge, badge_activity) = self
            .badges
            .iter()
            .find_map(|value| value.get_by_activity(activity))?;
        let progress = badge_activity.get_progress(&activity.attributes);
        let levels: Vec<&BadgeLevel> = badge
            .levels
            .iter()
            .filter(|value| value.target_count <= progress)
            .collect();
        Some((badge, progress, levels))
    }

    pub fn get_modifier_entry(
        &self,
        name: &str,
        value: &str,
    ) -> Option<(&MatchModifier, &MatchModifierEntry)> {
        let modifier = self
            .modifiers
            .iter()
            // Find the specific modifier by name
            .find(|modifier| modifier.name.eq(name))?;

        // Find the modifier value by the desired value
        let value = modifier.values.iter().find(|entry| entry.name.eq(value))?;

        Some((modifier, value))
    }
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

    pub currency: CurrencyName,

    pub activities: Vec<ActivityDescriptor>,
    pub levels: Vec<BadgeLevel>,
}

impl Badge {
    /// Finds a badge activity details using the provided mission activity
    /// matches against the name and attribute filter
    pub fn get_by_activity(
        &self,
        activity: &MissionActivity,
    ) -> Option<(&Self, &ActivityDescriptor)> {
        self.activities.iter().find_map(|value| {
            if value.matches(activity) {
                Some((self, value))
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDescriptor {
    pub activity_name: String,
    pub filter: Map<String, Value>,
    pub increment_progress_by: String,
}

impl ActivityDescriptor {
    pub fn matches(&self, activity: &MissionActivity) -> bool {
        self.activity_name.eq(&activity.name) && self.matches_filter(&activity.attributes.extra)
    }

    /// Checks if the badge activity filter matches the attributes of
    /// the provided activity
    pub fn matches_filter(&self, attributes: &Map<String, Value>) -> bool {
        for (key, value) in self.filter.iter() {
            let right = match attributes.get(key) {
                Some(value) => value,
                None => return false,
            };
            if value.ne(right) {
                return false;
            }
        }

        true
    }

    /// Obtains the progress value from the provided attributes based
    /// on the progress increment target
    pub fn get_progress(&self, attributes: &MissionActivityAttributes) -> u32 {
        match self.increment_progress_by.as_str() {
            "count" => attributes.count,
            "score" => attributes.score,
            _ => 0,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifier {
    pub name: String,
    pub values: Vec<MatchModifierEntry>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifierEntry {
    pub name: String,
    pub xp_data: Option<ModifierData>,
    pub currency_data: HashMap<CurrencyName, ModifierData>,
    pub custom_attributes: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierData {
    pub flat_amount: u32,
    pub additive_multiplier: f32,
}

impl ModifierData {
    /// Returns the amount that should be added based on
    /// the old value with the modifier
    pub fn get_amount(&self, old_value: u32) -> u32 {
        let adative_value = (old_value as f32 * self.additive_multiplier).trunc() as u32;
        self.flat_amount + adative_value
    }
}
