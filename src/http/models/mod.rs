use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Serialize;

pub mod auth;
pub mod client;

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
