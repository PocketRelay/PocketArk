use std::{error::Error, fmt::Debug};

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::error;
use sea_orm::{DbErr, TransactionError};
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

pub type HttpResult<T> = Result<Json<T>, RawHttpError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWithCount<V>
where
    V: Debug + Sized + Serialize + 'static,
{
    pub total_count: usize,
    pub list: &'static [V],
}

impl<V> ListWithCount<V>
where
    V: Debug + Sized + Serialize + 'static,
{
    pub fn new(list: &'static [V]) -> Self {
        Self {
            total_count: list.len(),
            list,
        }
    }
}

/// Dynamic error type for handling many error types
pub struct DynHttpError {
    /// The dynamic error cause
    error: Box<dyn HttpError>,
}

/// Trait implemented by errors that can be converted into [HttpError]s
/// and used as error responses
trait HttpError: Error + 'static {
    /// Used to create the status code for an error created
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Used to create the reason string
    fn reason(&self) -> String {
        self.to_string()
    }
}

impl HttpError for DbErr {
    fn reason(&self) -> String {
        // Database errors shouldn't be visible to users
        "Server error".to_string()
    }
}

impl<E> From<E> for DynHttpError
where
    E: HttpError,
{
    fn from(value: E) -> Self {
        DynHttpError {
            error: Box::new(value),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawHttpError {
    #[serde(skip)]
    pub status: StatusCode,
    pub reason: String,
    pub cause: Option<String>,
    pub stack_trace: Option<String>,
    pub trace_id: Option<String>,
}

impl RawHttpError {
    pub fn new(reason: &str, status: StatusCode) -> Self {
        Self {
            status,
            reason: reason.to_string(),
            cause: None,
            stack_trace: None,
            trace_id: None,
        }
    }
    pub fn new_owned(reason: String, status: StatusCode) -> Self {
        Self {
            status,
            reason,
            cause: None,
            stack_trace: None,
            trace_id: None,
        }
    }
}

impl From<DbErr> for RawHttpError {
    fn from(err: DbErr) -> Self {
        error!("Database error: {}", err);
        Self::new("Server error", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl<E> From<TransactionError<E>> for RawHttpError
where
    E: Error + Into<RawHttpError>,
{
    fn from(value: TransactionError<E>) -> Self {
        match value {
            TransactionError::Connection(value) => value.into(),
            TransactionError::Transaction(value) => value.into(),
        }
    }
}

impl IntoResponse for RawHttpError {
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
