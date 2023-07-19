use std::{collections::HashMap, process::exit};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

pub const MATCH_BADGE_DEFINITIONS: &str = include_str!("../../resources/data/matchBadges.json");
pub const MATCH_MODIFIER_DEFINITIONS: &str =
    include_str!("../../resources/data/matchModifiers.json");

pub struct MatchDataService {
    pub badges: Vec<Badge>,
    pub modifiers: Vec<MatchModifier>,
}

impl MatchDataService {
    pub fn load() -> Self {
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

    pub fn get_by_activity(&self, activity: &Uuid) -> Option<&Badge> {
        self.badges.iter().find(|value| {
            value
                .activities
                .iter()
                .any(|value| value.activity_name.eq(activity))
        })
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

impl BadgeActivity {
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

        return true;
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
    pub currency_data: HashMap<String, ModifierData>,
    pub custom_attributes: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierData {
    pub flat_amount: u32,
    pub additive_multiplier: f64,
}
