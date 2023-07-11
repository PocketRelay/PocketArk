use axum::response::{IntoResponse, Response};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};
use uuid::Uuid;

/// GET /challenges
pub async fn get_challenges() -> Response {
    let mut resp = include_str!("../../resources/defs/min/challenges.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /challenges/user
pub async fn get_user_challenges() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_User_Challenges-1688700271729.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
/// GET /challenges/categories
pub async fn get_challenge_categories() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Challenge_Categories-1689040191079.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
