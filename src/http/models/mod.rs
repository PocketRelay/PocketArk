use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::error;
use sea_orm::DbErr;
use serde::Serialize;

pub mod auth;
pub mod challenge;
pub mod character;
pub mod client;
pub mod inventory;
pub mod leaderboard;
pub mod mission;
pub mod qos;
pub mod store;
pub mod strike_teams;
pub mod telemetry;
pub mod user_match;

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

impl HttpError {
    pub fn new(reason: &str, status: StatusCode) -> Self {
        Self {
            status,
            reason: reason.to_string(),
            cause: None,
            stack_trace: None,
            trace_id: None,
        }
    }
}

impl From<DbErr> for HttpError {
    fn from(err: DbErr) -> Self {
        error!("Database error: {}", err);
        Self::new("Server error", StatusCode::INTERNAL_SERVER_ERROR)
    }
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
