use crate::http::models::telemetry::PinResponse;
use axum::Json;
use log::debug;

/// POST /pinEvents
///
/// Recieves telemetry messages from the client always responding
/// with an ok status
///
/// TODO: Log / save the messages sent to this endpoint (Its JSON just string is more readable)
pub async fn pin_events(req: String) -> Json<PinResponse> {
    debug!("Event pinned: {}", req);

    Json(PinResponse {
        status: "ok".to_string(),
    })
}
