use axum::response::{IntoResponse, Response};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};

use crate::http::models::RawJson;

/// Challenge category definitions
static CHALLENGE_CATEGORIES: &str = include_str!("../../resources/data/challengeCategories.json");

/// GET /challenges/categories
///
/// Intended to obtain a list of challenge categories but
/// seems to just return an empty response
pub async fn get_challenge_categories() -> RawJson {
    RawJson(CHALLENGE_CATEGORIES)
}

/// Challenge definitions
static CHALLENGES_DEFINITION: &str = include_str!("../../resources/data/challenges.json");

/// GET /challenges
///
/// Obtains a list of all the challenges that can be completed
pub async fn get_challenges() -> RawJson {
    RawJson(CHALLENGES_DEFINITION)
}

/// GET /challenges/user
///
/// Obtains a list of all the challenges the user has either
/// completed or has started.
pub async fn get_user_challenges() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_User_Challenges-1688700271729.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
