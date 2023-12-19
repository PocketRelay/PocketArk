use crate::database::entity::currency::CurrencyType;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, process::exit};
use uuid::Uuid;

use super::activity::{
    ActivityAttribute, ActivityDescriptor, ActivityEvent, ActivityName, AttributeName,
};

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
        activity: &ActivityEvent,
    ) -> Option<(&Badge, u32, Vec<&BadgeLevel>)> {
        // Find a badge with an activity that can be applied
        let (badge, badge_activity) = self.badges.iter().find_map(|badge| {
            badge
                .get_by_activity(activity)
                .map(|activity| (badge, activity))
        })?;

        // Get the activity progression
        let progress = activity.attribute_u32(&badge_activity.progress_key).ok()?;

        // Find all the badge levels that have been reached
        let levels: Vec<&BadgeLevel> = badge
            .levels
            .iter()
            .take_while(|level| level.target_count <= progress)
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

/// Type alias for a [Uuid] that represents the name of a [Badge]
pub type BadgeName = Uuid;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    /// Unique badge name
    pub name: BadgeName,
    /// Description of the badge
    pub description: String,
    /// Additional custom attributes
    pub custom_attributes: Map<String, Value>,
    /// Whether the badge can be awarded (Appears unused)
    pub enabled: bool,

    // Localization text
    pub i18n_title: Option<String>,
    pub i18n_description: Option<String>,
    pub loc_title: Option<String>,
    pub loc_description: Option<String>,

    /// The type of currency given as a reward for this badge
    pub currency: CurrencyType,

    /// [ActivityDescriptor]s describing how this badge can be awarded
    pub activities: Vec<ActivityDescriptor>,
    /// The different tiers / levels of this badge
    pub levels: Vec<BadgeLevel>,
}

impl Badge {
    /// Finds the [ActivityDescriptor] within this [Badge] that matches the
    /// provided `activity` if there is one available
    pub fn get_by_activity(&self, activity: &ActivityEvent) -> Option<&ActivityDescriptor> {
        self.activities
            .iter()
            .find(|descriptor| descriptor.matches(activity))
    }
}

/// Alias for a string representing the name of a [BadgeLevel]
pub type BadgeLevelName = String;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeLevel {
    /// Name of the badge level ("Bronze", "Silver", "Gold", "Platinum", ..etc)
    pub name: BadgeLevelName,
    /// Internal game path for the image used when displaying the badge
    pub img_path: Option<String>,
    /// Appears to be unused
    #[serde(rename = "imgURLFull")]
    pub img_url_full: Option<String>,
    /// The required progress count for the level to be reached
    pub target_count: u32,
    /// The total XP to give for completing this level
    pub xp_reward: u32,
    /// The total currency to give for completing this level the
    /// type of currency awarded is [Badge::currency]
    pub currency_reward: u32,
    /// Possibly item rewards? Haven't found this used yet
    pub rewards: Vec<Value>,
    /// Additional attributes on the badge (Appears to be unused)
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
    pub currency_data: HashMap<CurrencyType, ModifierData>,
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
