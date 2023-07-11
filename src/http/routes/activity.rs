use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};

/// POST /activity
pub async fn create_report() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Create_Activity_Report-1688700352069.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /activity/metadata
pub async fn get_metadata() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_Activity_Meta-1688700317788.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
