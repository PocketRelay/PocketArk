use serde_json::{Map, Value};

use crate::{database::entity::User, http::models::mission::MissionActivity, state::App};

use super::challenges::ChallengeProgressUpdate;

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
