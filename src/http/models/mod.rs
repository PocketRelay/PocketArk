use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use serde::Serialize;

pub mod auth;
pub mod character;
pub mod client;
pub mod inventory;
pub mod leaderboard;
pub mod mission;
pub mod store;
pub mod telemetry;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpError {
    #[serde(skip)]
    pub status: StatusCode,
    pub reason: String,
    pub cause: Option<String>,
    pub stack_trace: Option<String>,
    pub trace_id: Option<String>,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self)).into_response()
    }
}

/// Raw pre encoded JSON string response
pub struct RawJson(pub &'static str);

impl IntoResponse for RawJson {
    fn into_response(self) -> Response {
        let mut res = self.0.into_response();
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        res
    }
}
