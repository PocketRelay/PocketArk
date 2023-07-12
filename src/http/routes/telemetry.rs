use crate::http::models::telemetry::PinResponse;
use axum::Json;

/// POST /pinEvents
///
/// Recieves telemetry messages from the client always responding
/// with an ok status
///
/// TODO: Log / save the messages sent to this endpoint
pub async fn pin_events() -> Json<PinResponse> {
    Json(PinResponse {
        status: "ok".to_string(),
    })
}
