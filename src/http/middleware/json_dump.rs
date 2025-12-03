//! JSON extractor that dumps invalid JSON messages to the logs

use axum::extract::{FromRequest, Request};
use bytes::Bytes;
use hyper::StatusCode;
use log::{debug, error};
use serde::de::DeserializeOwned;

/// [axum::Json] extractor alternative for use in debug mode that dumps
/// incoming payloads to the debug log
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonDump<T>(pub T);

impl<T, S> FromRequest<S> for JsonDump<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|err| {
            error!("Failed to get request bytes: {}", err);
            StatusCode::BAD_REQUEST
        })?;

        debug!("Incoming JSON: {:?}", bytes);

        let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);

        let value = match serde_path_to_error::deserialize(deserializer) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to deserialize request: {}", err);

                return Err(StatusCode::BAD_REQUEST);
            }
        };

        Ok(JsonDump(value))
    }
}
