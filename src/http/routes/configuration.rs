use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};

static CONFIGURATION: &str = include_str!("../../resources/defs/min/configuration.json");

/// GET /configuration
pub async fn get_configuration() -> Response {
    let mut resp = CONFIGURATION.into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
