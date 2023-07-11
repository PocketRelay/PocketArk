use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};

/// POST /pinEvents
pub async fn pin_events() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Pin-1689040402347.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
