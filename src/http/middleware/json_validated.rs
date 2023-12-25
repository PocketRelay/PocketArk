//! JSON extractor that validates the underlying value
//! is valid using [validator::Validate]

use axum::{async_trait, extract::FromRequest, BoxError};
use bytes::Bytes;
use hyper::{body::HttpBody, Request, StatusCode};
use log::error;
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

use crate::http::models::{DynHttpError, HttpError};

/// [axum::Json] extractor alternative for use in debug mode that dumps
/// incoming payloads to the debug log
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonValidated<T: Validate>(pub T);

/// Error types that could be returned on rejection
#[derive(Debug, Error)]
pub enum RejectionError {
    /// Unable to load the content
    #[error("Content error")]
    BadContent,
    /// Failed to deserialize
    #[error(transparent)]
    Deserialize(serde_path_to_error::Error<serde_json::Error>),

    /// Failed validation
    #[error(transparent)]
    Validation(validator::ValidationErrors),
}

impl HttpError for RejectionError {
    fn status(&self) -> StatusCode {
        match self {
            RejectionError::BadContent
            | RejectionError::Deserialize(_)
            | RejectionError::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }
}

#[async_trait]
impl<T, S, B> FromRequest<S, B> for JsonValidated<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = DynHttpError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        // Get request byets
        let bytes = Bytes::from_request(req, state).await.map_err(|err| {
            error!("Failed to get request bytes: {}", err);
            RejectionError::BadContent
        })?;

        // Deserialize value
        let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);
        let value: T =
            serde_path_to_error::deserialize(deserializer).map_err(RejectionError::Deserialize)?;

        // Validate deserialized value
        value.validate().map_err(RejectionError::Validation)?;

        Ok(JsonValidated(value))
    }
}
