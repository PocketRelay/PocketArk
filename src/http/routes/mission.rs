use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

/// GET /mission/current
pub async fn current_mission() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_Current_Mission-1688700356655.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /user/mission/:id
pub async fn get_mission() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_Mission_Details-1688700361289.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// POST /user/mission/:id/start
pub async fn start_mission() -> Response {
    StatusCode::OK.into_response()
}

/// POST /user/mission/:id/finish
pub async fn finish_mission() -> Response {
    StatusCode::OK.into_response()
}
