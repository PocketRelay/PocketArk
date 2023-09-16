//! This module contains extensions that allow upgrading an HTTP
//! request into a Blaze steam

use axum::{
    extract::FromRequestParts,
    http::{Method, StatusCode},
    response::IntoResponse,
};
use futures::future::BoxFuture;
use hyper::{
    upgrade::{OnUpgrade, Upgraded},
    HeaderMap,
};
use log::debug;
use std::future::ready;
use thiserror::Error;

/// Errors that could occur while upgrading
#[derive(Debug, Error)]
pub enum BlazeUpgradeError {
    #[error("Cannot upgrade not GET requests")]
    UnacceptableMethod,
    #[error("Failed to upgrade connection")]
    FailedUpgrade,
    #[error("Cannot upgrade connection")]
    CannotUpgrade,
}

/// Extractor for initiated the upgrade process for a request
pub struct BlazeUpgrade {
    /// The upgrade handle
    on_upgrade: OnUpgrade,
    pub token: Box<str>,
}

/// HTTP request upgraded into a Blaze socket along with
/// extra information
pub struct BlazeSocket {
    /// The upgraded connection
    pub upgrade: Upgraded,
    pub token: Box<str>,
}

impl BlazeUpgrade {
    /// Upgrades the underlying hook returning the newly created socket
    pub async fn upgrade(self) -> Result<BlazeSocket, BlazeUpgradeError> {
        // Attempt to upgrade the connection
        let upgrade = match self.on_upgrade.await {
            Ok(value) => value,
            Err(_) => return Err(BlazeUpgradeError::FailedUpgrade),
        };

        Ok(BlazeSocket {
            upgrade,
            token: self.token,
        })
    }

    fn extract_auth(headers: &HeaderMap) -> Option<Box<str>> {
        let header = headers.get(HEADER_AUTH)?;
        let header = header.to_str().ok()?;
        Some(Box::from(header))
    }
}

/// Header for the Pocket Ark client authentication
const HEADER_AUTH: &str = "X-Pocket-Ark-Auth";

impl<S> FromRequestParts<S> for BlazeUpgrade
where
    S: Send + Sync,
{
    type Rejection = BlazeUpgradeError;

    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut axum::http::request::Parts,
        _state: &'b S,
    ) -> BoxFuture<'c, Result<Self, Self::Rejection>>
    where
        'a: 'c,
        'b: 'c,
        Self: 'c,
    {
        debug!("Attempting upgrade of blaze client");

        // Ensure the method is GET
        if parts.method != Method::GET {
            return Box::pin(ready(Err(BlazeUpgradeError::UnacceptableMethod)));
        }

        // Get the upgrade hook
        let on_upgrade = match parts.extensions.remove::<OnUpgrade>() {
            Some(value) => value,
            None => return Box::pin(ready(Err(BlazeUpgradeError::CannotUpgrade))),
        };

        let headers = &parts.headers;

        // Get the client auth
        let token: Box<str> = match BlazeUpgrade::extract_auth(headers) {
            Some(value) => value,
            None => return Box::pin(ready(Err(BlazeUpgradeError::CannotUpgrade))),
        };

        Box::pin(ready(Ok(Self { on_upgrade, token })))
    }
}

impl IntoResponse for BlazeUpgradeError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
