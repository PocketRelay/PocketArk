use crate::{
    http::{
        middleware::{JsonDump, user::Auth},
        models::{DynHttpError, RawJsonBytes, activity::HttpActivityResult},
    },
    resources::Resources,
    services::activity::result::ActivityResult,
};
use anyhow::Context;
use axum::Json;
use serde_json::Value;
use tracing::debug;

/// POST /activity
///
/// This endpoint receives requests whenever in game activities
/// from the activity metadata definitions are completed. The request
/// contains details about the activity
pub async fn create_report(
    Auth(user): Auth,
    JsonDump(req): JsonDump<Value>,
) -> Json<HttpActivityResult> {
    debug!("Activity reported: {} {}", user.username, req);

    // TODO: actually handle activities

    Json(HttpActivityResult::from(ActivityResult::default()))
}

const ACTIVITY_METADATA_RESOURCE: &str = "activityMetadata.json";

/// GET /activity/metadata
///
/// Obtains the definitions of activities that can happen within a game.
/// When these activities happen a report is posted to `create_report`
pub async fn get_metadata() -> Result<RawJsonBytes, DynHttpError> {
    let data = Resources::get_resource_bytes(ACTIVITY_METADATA_RESOURCE)
        .await
        .context("failed to load configuration")?;
    Ok(RawJsonBytes(data))
}

/// PUT /wv/playthrough/0
///
/// Server receives updates about the players
/// singleplayer playthrough choices
pub async fn update_playthrough(req: String) {
    debug!("Update playthrough {}", req);
}
