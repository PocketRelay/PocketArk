use axum::Json;
use log::debug;
use serde_json::{Map, Value};

use crate::http::{
    middleware::user::Auth,
    models::{inventory::ActivityResult, RawJson},
};

/// POST /activity
///
/// This endpoint recieves requests whenever in game activities
/// from the activity metadata definitions are completed. The request
/// contains details about the activity
pub async fn create_report(Auth(user): Auth, req: String) -> Json<ActivityResult> {
    debug!("Activity reported: {} {}", user.username, req);
    Json(ActivityResult {
        previous_xp: 0,
        xp: 0,
        xp_gained: 0,
        previous_level: 0,
        level: 0,
        level_up: false,
        character_class_name: None,
        challenges_updated_count: 0,
        challenges_completed_count: 0,
        challenges_updated: vec![],
        updated_challenge_ids: vec![],
        news_triggered: 0,
        currencies: vec![],
        currency_earned: vec![],
        items_earned: vec![],
        item_definitions: vec![],
        entitlements_granted: vec![],
        prestige_progression_map: Map::new(),
    })
}

/// Definition of different activities that can happen within a game.
static ACTIVITY_METADATA_DEFINITION: &str =
    include_str!("../../resources/data/activityMetadata.json");

/// GET /activity/metadata
///
/// Obtains the definitions of activities that can happen within a game.
/// When these activities happen a report is posted to `create_report`
pub async fn get_metadata() -> RawJson {
    RawJson(ACTIVITY_METADATA_DEFINITION)
}

/// PUT /wv/playthrough/0
///
/// Server recieves updates about the players
/// singleplayer playthrough choices
pub async fn update_playthrough(Json(req): Json<Value>) {
    debug!("Update playthrough {:?}", req);
}
