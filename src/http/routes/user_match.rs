use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};

/// GET /user/match/badges
pub async fn get_badges() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_User_Match_Badges-1688700344615.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /user/match/modifiers
pub async fn get_modifiers() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_User_Match_Modifiers-1688700322703.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
