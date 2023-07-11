use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

/// PUT /presence/session
pub async fn update_session() -> Response {
    StatusCode::NO_CONTENT.into_response()
}
