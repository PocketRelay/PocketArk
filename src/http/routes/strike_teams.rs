use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

/// GET /striketeams
pub async fn get() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_Strike_Teams-1688700334138.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /striketeams/successRate
pub async fn get_success_rate() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Strike_Team_Success_Rate-1688700327687.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
