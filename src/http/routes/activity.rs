use crate::{
    http::{
        middleware::{user::Auth, JsonDump},
        models::RawJson,
    },
    services::activity::ActivityResult,
};
use axum::Json;
use log::debug;
use serde_json::Value;

/// POST /activity
///
/// This endpoint recieves requests whenever in game activities
/// from the activity metadata definitions are completed. The request
/// contains details about the activity
pub async fn create_report(
    Auth(user): Auth,
    JsonDump(req): JsonDump<Value>,
) -> Json<ActivityResult> {
    debug!("Activity reported: {} {}", user.username, req);

    // TODO: actually handle activities

    Json(ActivityResult::default())
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
pub async fn update_playthrough(req: String) {
    debug!("Update playthrough {}", req);
}
