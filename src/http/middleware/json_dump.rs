//! JSON extractor that dumps invalid JSON messages to the logs

use axum::{async_trait, extract::FromRequest, BoxError};
use bytes::Bytes;
use hyper::{body::HttpBody, Request, StatusCode};
use log::{debug, error};
use serde::de::DeserializeOwned;

/// [axum::Json] extractor alternative for use in debug mode that dumps
/// incoming payloads to the debug log
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonDump<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for JsonDump<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
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
