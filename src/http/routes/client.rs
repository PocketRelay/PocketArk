//! This module contains HTTP routes and logic used between the server
//! and the PocketArk client

use axum::{
    body::Empty,
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header, http::HeaderValue, StatusCode};
use log::{debug, error};
use serde::Serialize;
use tokio_util::codec::Framed;

use crate::{
    blaze::{pk::packet::PacketCodec, session::Session},
    http::middleware::upgrade::BlazeUpgrade,
};

#[derive(Serialize)]
pub struct ServerDetails {
    /// Identifier used to ensure the server is a Pocket Relay server
    ident: &'static str,
    /// The server version
    version: &'static str,
}

/// GET /ark/client/details
pub async fn details() -> Json<ServerDetails> {
    Json(ServerDetails {
        ident: "POCKET_ARK_SERVER",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /ark/client/auth
pub async fn authenticate() {}

/// GET /ark/client/upgrade
pub async fn upgrade(upgrade: BlazeUpgrade) -> Response {
    tokio::spawn(async move {
        let socket = match upgrade.upgrade().await {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to upgrade blaze socket: {}", err);
                return;
            }
        };
        // TODO: Validate authentication

        let session = Session::new(Framed::new(socket.upgrade, PacketCodec), socket.host_target);

        debug!("New session: {}", &session.id);

        session.handle().await;
    });

    let mut response = Empty::new().into_response();
    // Use the switching protocols status code
    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

    let headers = response.headers_mut();
    // Add the upgraidng headers
    headers.insert(header::CONNECTION, HeaderValue::from_static("upgrade"));
    headers.insert(header::UPGRADE, HeaderValue::from_static("blaze"));

    response
}