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
use tokio::io::split;
use tokio_util::codec::{Framed, FramedRead, FramedWrite};

use crate::{
    blaze::{
        packet::PacketCodec,
        routes::{handle_session, Session},
    },
    http::{middleware::upgrade::BlazeUpgrade, models::client::*},
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

        // Obtain a session ID
        let session_id = uuid::Uuid::new_v4();

        let session = Session {
            io: Framed::new(socket.upgrade, PacketCodec),
            id: session_id,
            host_target: socket.host_target,
        };

        debug!("New session: {}", session_id);

        handle_session(session).await;
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
