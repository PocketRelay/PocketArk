use serde_json::{Map, Value};

use crate::{database::entity::User, http::models::mission::MissionActivity};

pub struct ActivityService {}

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

    pub async fn process_activity(&self, user: &User, activity: MissionActivity) {
        match activity.name.as_str() {
            Self::ITEM_CONSUMED => {}
            Self::BADGE_EARNED => {}
            Self::ARTICLE_PURCHASED => {}
            Self::MISSION_FINISHED => {}
            Self::EQUIPMENT_ATTACHMENT_UPDATED => {}
            Self::EQUIPMENT_UPDATED => {}
            Self::SKILL_PURCHASED => {}
            Self::CHARACTER_LEVEL_UP => {}
            Self::STRIKE_TEAM_RECRUITED => {}
            name => {}
        }
    }
}
