//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use super::items::ItemDefinition;
use crate::{
    database::entity::{Currency, InventoryItem},
    state::App,
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use serde_with::skip_serializing_none;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub struct ActivityService;

#[allow(unused)]
impl ActivityService {
    // Hardcoded activity types
    pub const ITEM_CONSUMED: &'static str = "_itemConsumed";
    pub const BADGE_EARNED: &'static str = "_badgeEarned";
    pub const ARTICLE_PURCHASED: &'static str = "_articlePurchased";
    pub const MISSION_FINISHED: &'static str = "_missionFinished";
    pub const EQUIPMENT_ATTACHMENT_UPDATED: &'static str = "_equipmentAttachmentUpdated";
    pub const EQUIPMENT_UPDATED: &'static str = "_equipmentUpdated";
    pub const SKILL_PURCHASED: &'static str = "_skillPurchased";
    pub const CHARACTER_LEVEL_UP: &'static str = "_characterLevelUp";
    pub const STRIKE_TEAM_RECRUITED: &'static str = "_strikeTeamRecruited";
}

/// Represents the name for an activity, contains built in
/// server activity types along with the [Uuid] variant for
/// runtime defined activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityName {
    /// Item was consumed
    #[serde(rename = "_itemConsumed")]
    ItemConsumed,
    /// Badge was earned on game completion
    #[serde(rename = "_badgeEarned")]
    BadgeEarned,
    /// Article was purchased from the store
    #[serde(rename = "_articlePurchased")]
    ArticlePurchased,
    /// Mission was finished
    ///
    /// Known attributes:
    /// - percentComplete (number)
    /// - missionTypeName (string uuid)
    /// - count (number)
    #[serde(rename = "_missionFinished")]
    MissionFinished,
    /// Mission was finished by a strike team
    ///
    /// Known attributes:
    /// - success (string boolean)
    /// - count (number)
    #[serde(rename = "_strikeTeamMissionFinished")]
    StrikeTeamMissionFinished,
    /// Equipment was updated
    ///
    /// Known attributes:
    /// - slot (string)
    /// - count (number)
    /// - stackSize (number)
    #[serde(rename = "_equipmentUpdated")]
    EquipmentUpdated,
    /// Equipment attachments were updated
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_equipmentAttachmentUpdated")]
    EquipmentAttachmentUpdated,
    /// Skills were purchased
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_skillPurchased")]
    SkillPurchased,
    /// Character was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - characterClass (string uuid)
    /// - count (number)
    #[serde(rename = "_characterLevelUp")]
    CharacterLevelUp,
    /// Prestige was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - count (number)
    #[serde(rename = "_prestigeLevelUp")]
    PrestigeLevelUp,
    /// Pathfinder rating has changed
    ///
    /// Known attributes
    /// - pathfinderRatingDelta (number)
    #[serde(rename = "_pathfinderRatingUpdated")]
    PathfinderRatingUpdated,
    /// Strike team was recruited
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_strikeTeamRecruited")]
    StrikeTeamRecruited,
    /// Activity represented by a [Uuid] these events can be
    /// published by clients
    #[serde(untagged)]
    Named(Uuid),
}

/// Represents a published activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// The name of the activity event
    pub name: ActivityName,
    /// Data attributes associated with this activity event
    pub attributes: HashMap<AttributeName, ActivityAttribute>,
}

/// Type alias for a string representing an attribute name
pub type AttributeName = String;

/// Represents an attribute within an [ActivityEvent]. These
/// can be numbers or strings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityAttribute {
    /// String value
    String(String),
    /// Number value
    Number(Number),
}

impl ActivityEvent {
    /// Obtains the current progress from the activity attributes
    /// based on the provided `key`.
    ///
    /// Returns [None] if the progress value was invalid
    /// or missing
    pub fn get_progress(&self, key: &str) -> Option<u32> {
        self.attributes
            // Get the progress value
            .get(key)
            // Take the number progress value
            .and_then(|value| match value {
                ActivityAttribute::String(_) => None,
                ActivityAttribute::Number(value) => value.as_u64(),
            })
            // Don't need full precision of 64bit only need 32bit
            .map(|value| value as u32)
    }

    /// Obtains the score from the mission activity if it
    /// is present within the attributes
    #[inline]
    pub fn get_score(&self) -> Option<u32> {
        self.get_progress("score")
    }

    /// Checks if this activity `attributes` match the provided filter
    pub fn matches_filter(&self, filter: &HashMap<AttributeName, ActivityFilter>) -> bool {
        filter
            .iter()
            // Ensure all attributes match
            .all(|(key, filter)| {
                self.attributes
                    .get(key)
                    // Ensure the value exists and matches
                    .is_some_and(|value| filter.matches(value))
            })
    }
}

/// Describes an activity that can be used to track progress
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDescriptor {
    /// Name of the [ActivityEvent] this descriptor is for
    /// (Can be a [Uuid] or just text such as: "_itemConsumed")
    pub activity_name: ActivityName,
    /// Filtering based on the [ActivityEvent::attributes] for
    /// whether the activity is applicable
    pub filter: HashMap<AttributeName, ActivityFilter>,
    /// The key into [ActivityEvent::attributes] that should be
    /// used for tracking activity progress
    #[serde(rename = "incrementProgressBy")]
    pub progress_key: String,
}

/// Enum for different ways an activity can be filtered against
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityFilter {
    /// Direct value comparison
    Value(ActivityAttribute),
    /// Not equal comparison
    NotEqual {
        /// The value to compare not equal against
        #[serde(rename = "$ne")]
        ne: ActivityAttribute,
    },
}

impl ActivityFilter {
    pub fn matches(&self, other: &ActivityAttribute) -> bool {
        match self {
            Self::Value(value) => value.eq(other),
            Self::NotEqual { ne } => ne.ne(other),
        }
    }
}

impl ActivityDescriptor {
    /// Checks if the provided `activity` matches this descriptor
    pub fn matches(&self, activity: &ActivityEvent) -> bool {
        self.activity_name.eq(&activity.name) && activity.matches_filter(&self.filter)
    }
}

/// Represents the result produced from processing an [ActivityEvent]
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResult {
    #[serde(flatten)]
    pub xp: ActivityXpDetails,

    #[serde(flatten)]
    pub level: ActivityLevelDetails,

    pub character_class_name: Option<Uuid>,

    #[serde(flatten)]
    pub challenge: ActivityChallengeDetails,

    pub news_triggered: u32,
    /// The new total currency amounts
    pub currencies: Vec<Currency>,
    /// The amounts that were earned
    pub currency_earned: Vec<Currency>,

    #[serde(flatten)]
    pub items: ActivityItemDetails,

    pub entitlements_granted: Vec<Value>,
    #[serde(rename = "prestigeProgressionMap")]
    pub prestige: PrestigeProgression,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct ActivityItemDetails {
    #[serde(rename = "itemsEarned")]
    pub earned: Vec<InventoryItem>,
    #[serde(rename = "itemDefinitions")]
    pub definitions: Vec<&'static ItemDefinition>,
}

#[derive(Debug, Default, Serialize)]
pub struct ActivityChallengeDetails {
    #[serde(rename = "challengesUpdatedCount")]
    pub updated_count: u32,
    #[serde(rename = "challengesCompletedCount")]
    pub completed_count: u32,
    #[serde(rename = "challengesUpdated")]
    pub challenges_updated: BTreeMap<String, ChallengeUpdate>,
    #[serde(rename = "updatedChallengeIds")]
    pub updated_ids: Vec<Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLevelDetails {
    pub previous_level: u32,
    pub level: u32,
    pub level_up: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityXpDetails {
    pub xp: u32,
    pub previous_xp: u32,
    pub xp_gained: u32,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrestigeProgression {
    pub before: Option<HashMap<Uuid, PrestigeData>>,
    pub after: Option<HashMap<Uuid, PrestigeData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeData {
    pub name: Uuid,
    pub level: u32,
    pub xp: u32,
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
