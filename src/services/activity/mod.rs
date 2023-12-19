//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use super::items::ItemDefinition;
use crate::{
    database::entity::{
        challenge_progress::{ChallengeCounterName, ChallengeId},
        Currency, InventoryItem,
    },
    state::App,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_json::{Number, Value};
use serde_with::skip_serializing_none;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub struct ActivityService;

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
    /// Checks whether the provided [ActivityAttribute] matches this filter
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
#[derive(Debug, Default)]
pub struct ActivityResult {
    /// The previous character XP
    pub previous_xp: u32,
    /// The current character XP
    pub current_xp: u32,
    /// The amount of XP gained
    pub gained_xp: u32,

    /// The previous character level
    pub previous_level: u32,
    /// The current character level
    pub current_level: u32,

    /// Present in strike team activity resolves
    pub character_class_name: Option<Uuid>,

    /// The number of challenges completed
    pub challeges_completed: u32,
    /// Challenges that were updates
    pub challenges_updated: Vec<ChallengeUpdated>,

    /// Unknown field
    pub news_triggered: u32,
    /// The currrent currency amounts that the player has
    pub currencies: Vec<Currency>,
    /// The different currency amounts that were earned
    pub currency_earned: Vec<Currency>,

    /// Items that were earned from the activity
    pub items_earned: Vec<InventoryItem>,
    /// Definitions for the items from `items_earned`
    pub item_definitions: Vec<&'static ItemDefinition>,

    /// Entitlements that were granted from the activity
    ///
    /// TODO: Haven't encounted a value for this yet so its untyped
    pub entitlements_granted: Vec<Value>,

    /// Prestige progression that resulted from the activity
    pub prestige_progression: PrestigeProgression,
}

impl Serialize for ActivityResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_struct("ActivityResult", 18)?;
        value.serialize_field("previousXp", &self.previous_xp)?;
        value.serialize_field("xp", &self.current_xp)?;
        value.serialize_field("xpGained", &self.gained_xp)?;

        value.serialize_field("previousLevel", &self.previous_level)?;
        value.serialize_field("level", &self.current_level)?;
        value.serialize_field("levelUp", &(self.current_level != self.previous_level))?;

        if let Some(character_class_name) = &self.character_class_name {
            value.serialize_field("characterClassName", character_class_name)?;
        }

        value.serialize_field("challengesUpdatedCount", &self.challenges_updated.len())?;
        value.serialize_field("challengesCompletedCount", &self.challeges_completed)?;
        value.serialize_field("challengesUpdated", &self.challenges_updated)?;

        /// Collect the updated challenge IDs for serialization
        let challenge_ids: Vec<ChallengeId> = self
            .challenges_updated
            .iter()
            .map(|value| value.challenge_id)
            .collect();

        value.serialize_field("updatedChallengeIds", &challenge_ids)?;
        value.serialize_field("newsTriggered", &self.news_triggered)?;
        value.serialize_field("currencies", &self.currencies)?;
        value.serialize_field("currencyEarned", &self.currency_earned)?;
        value.serialize_field("itemsEarned", &self.items_earned)?;
        value.serialize_field("itemDefinitions", &self.item_definitions)?;
        value.serialize_field("entitlementsGranted", &self.entitlements_granted)?;
        value.serialize_field("prestigeProgressionMap", &self.prestige_progression)?;
        value.end()
    }
}

/// Type alias for a [Uuid] representing the name of a prestige level table
pub type PrestigeName = Uuid;

/// Represents the difference between
#[derive(Debug, Clone, Default, Serialize)]
pub struct PrestigeProgression {
    /// The previous prestige data
    pub before: HashMap<PrestigeName, PrestigeData>,
    /// The new prestige data
    pub after: HashMap<PrestigeName, PrestigeData>,
}

/// Prestige data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeData {
    /// The name of the prestige level table
    pub name: PrestigeName,
    /// The prestige current level
    pub level: u32,
    /// The prestige current xp
    pub xp: u32,
}

/// Represents a challenge that was updated
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdated {
    /// The ID of the challenge that was updated
    pub challenge_id: ChallengeId,
    /// Counters that were updated
    pub counters: Vec<ChallengeUpdateCounter>,
    /// The change of status for the challenge update
    pub status_change: ChallengeStatusChange,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeStatusChange {
    /// Notifying the creation of the challenge progress
    Notify,
    /// An existing challenge progress changes
    Changed,
}

/// Represents a challenge counter that was updated
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdateCounter {
    /// The name of the counter that was updated
    pub name: ChallengeCounterName,
    /// The new counter value
    pub current_count: u32,
}
