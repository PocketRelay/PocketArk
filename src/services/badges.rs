use super::{
    activity::{ActivityDescriptor, ActivityEvent},
    i18n::{I18n, I18nDescription, I18nTitle, Localized},
};
use crate::database::entity::currency::CurrencyType;
use anyhow::Context;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::sync::OnceLock;
use uuid::Uuid;

/// Badge definitions (20)
const BADGE_DEFINITIONS: &str = include_str!("../resources/data/matchBadges.json");

/// Type alias for a [Uuid] that represents the name of a [Badge]
pub type BadgeName = Uuid;
/// Alias for a string representing the name of a [BadgeLevel]
pub type BadgeLevelName = String;

pub struct Badges {
    pub values: Vec<Badge>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<Badges> = OnceLock::new();

impl Badges {
    /// Gets a static reference to the global [Badges] collection
    pub fn get() -> &'static Badges {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let values: Vec<Badge> = serde_json::from_str(BADGE_DEFINITIONS)
            .context("Failed to load match badge definitions")?;

        debug!("Loaded {} badge definition(s)", values.len(),);

        Ok(Self { values })
    }

    pub fn by_activity(&self, activity: &ActivityEvent) -> Option<(&Badge, u32, Vec<&BadgeLevel>)> {
        // Find a badge with an activity that can be applied
        let (badge, badge_activity) = self.values.iter().find_map(|badge| {
            badge
                .by_activity(activity)
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
}

/// Represents a badge that can be earned while playing a match
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    /// Unique badge name
    pub name: BadgeName,
    /// Description of the badge
    pub description: String,
    /// Additional custom attributes
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
    /// Whether the badge can be awarded (Appears unused)
    pub enabled: bool,

    /// The type of currency given as a reward for this badge
    pub currency: CurrencyType,

    /// [ActivityDescriptor]s describing how this badge can be awarded
    pub activities: Vec<ActivityDescriptor>,
    /// The different tiers / levels of this badge
    pub levels: Vec<BadgeLevel>,

    /// Badge title
    #[serde(flatten)]
    pub i18n_title: I18nTitle,
    /// Badge description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

impl Badge {
    /// Finds the [ActivityDescriptor] within this [Badge] that matches the
    /// provided `activity` if there is one available
    pub fn by_activity(&self, activity: &ActivityEvent) -> Option<&ActivityDescriptor> {
        self.activities
            .iter()
            .find(|descriptor| descriptor.matches(activity))
    }
}

impl Localized for Badge {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_title.localize(i18n);
        self.i18n_description.localize(i18n);
    }
}

/// Represents a level of a badge
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeLevel {
    /// Name of the badge level ("", "Bronze", "Silver", "Gold", "Platinum", ..etc)
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
    pub rewards: Vec<serde_json::Value>,
    /// Additional attributes on the badge (Appears to be unused)
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod test {
    use super::Badges;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = Badges::load().unwrap();
    }
}
