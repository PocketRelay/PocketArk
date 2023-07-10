use axum::response::{IntoResponse, Response};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};
use uuid::Uuid;

static CHALLENGES: &str = include_str!("../../resources/defs/min/challenges.json");

/// GET /challenges
async fn get_challenges() -> Response {
    let mut resp = CHALLENGES.into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /challenges/user
async fn get_user_challenges() {}
