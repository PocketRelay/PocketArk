//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use super::items::ItemDefinition;
use crate::{
    database::entity::{Currency, InventoryItem},
    http::models::mission::MissionActivity,
    state::App,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub attributes: HashMap<String, serde_json::Value>,
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
