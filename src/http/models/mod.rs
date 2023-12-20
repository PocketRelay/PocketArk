use std::{
    error::Error,
    fmt::{Debug, Display},
};

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::error;
use sea_orm::{DbErr, TransactionError};
use serde::{ser::SerializeStruct, Serialize};

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

/// Type alias for dynamic error handling and JSON responses
pub type HttpResult<T> = Result<Json<T>, DynHttpError>;

/// Dynamic error type for handling many error types
pub struct DynHttpError {
    /// The dynamic error cause
    pub error: Box<dyn HttpError>,
}

impl Debug for DynHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl Display for DynHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl std::error::Error for DynHttpError {}

/// Handles converting the error into a response (Also logs the error before conversion)
impl IntoResponse for DynHttpError {
    fn into_response(self) -> Response {
        /// Handler for handling error responses and logging of [DynHttpError]s
        error!("{:?}: {}", &self, &self);

        // Create the response body
        let body = Json(RawHttpError {
            reason: self.error.reason(),
            cause: None,
            stack_trace: None,
            trace_id: None,
        });
        let status = self.error.status();

        (status, body).into_response()
    }
}

/// Trait implemented by errors that can be converted into [HttpError]s
/// and used as error responses
pub trait HttpError: Error + 'static {
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

// Handle storing dynamic typed errors
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

// Handle acceptable transaction errors
impl<E> HttpError for TransactionError<E>
where
    E: HttpError,
{
    fn reason(&self) -> String {
        match self {
            TransactionError::Connection(err) => err.reason(),
            TransactionError::Transaction(err) => err.reason(),
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            TransactionError::Connection(err) => err.status(),
            TransactionError::Transaction(err) => err.status(),
        }
    }
}

/// HTTP error JSON format for serializing responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawHttpError {
    pub reason: String,
    pub cause: Option<String>,
    pub stack_trace: Option<String>,
    pub trace_id: Option<String>,
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
