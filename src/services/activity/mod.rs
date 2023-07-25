use serde::Serialize;
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use super::{challenges::ChallengeProgressUpdate, items::ItemDefinition};
use crate::{
    database::entity::{Currency, InventoryItem},
    http::models::mission::MissionActivity,
    state::App,
};

pub struct ActivityService {}

#[allow(unused)]
impl ActivityService {
    // Hardcoded activity types
    pub const ITEM_CONSUMED: &str = "_itemConsumed";
    pub const BADGE_EARNED: &str = "_badgeEarned";
    pub const ARTICLE_PURCHASED: &str = "_articlePurchased";
    pub const MISSION_FINISHED: &str = "_missionFinished";
    pub const EQUIPMENT_ATTACHMENT_UPDATED: &str = "_equipmentAttachmentUpdated";
    pub const EQUIPMENT_UPDATED: &str = "_equipmentUpdated";
    pub const SKILL_PURCHASED: &str = "_skillPurchased";
    pub const CHARACTER_LEVEL_UP: &str = "_characterLevelUp";
    pub const STRIKE_TEAM_RECRUITED: &str = "_strikeTeamRecruited";

    pub fn new() -> Self {
        Self {}
    }

    pub fn process_activity(&self, activity: &MissionActivity) -> Option<ChallengeProgressUpdate> {
        let services = App::services();
        let (definition, counter, descriptor) = services.challenges.get_by_activity(activity)?;
        let progress = descriptor.get_progress(&activity.attributes);
        Some(ChallengeProgressUpdate {
            progress,
            counter,
            definition,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResult {
    pub previous_xp: u32,
    pub xp: u32,
    pub xp_gained: u32,
    pub previous_level: u32,
    pub level: u32,
    pub level_up: bool,
    pub character_class_name: Option<Uuid>,
    pub challenges_updated_count: u32,
    pub challenges_completed_count: u32,
    pub challenges_updated: Vec<Value>,
    pub updated_challenge_ids: Vec<Value>,
    pub news_triggered: u32,
    pub currencies: Vec<Currency>,
    pub currency_earned: Vec<Currency>,
    pub items_earned: Vec<InventoryItem>,
    pub item_definitions: Vec<&'static ItemDefinition>,
    pub entitlements_granted: Vec<Value>,
    pub prestige_progression_map: Map<String, Value>,
}
