use crate::http::models::RawJson;

/// Definition of different activities that can happen within a game.
static ACTIVITY_REPORT_RESULT: &str =
    include_str!("../../resources/data/activityReportResult.json");

/// POST /activity
///
/// This endpoint recieves requests whenever in game activities
/// from the activity metadata definitions are completed. The request
/// contains details about the activity
pub async fn create_report() -> RawJson {
    RawJson(ACTIVITY_REPORT_RESULT)
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
