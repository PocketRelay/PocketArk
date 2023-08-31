//! This module contains HTTP routes and logic used between the server
//! and the PocketArk client

use axum::{
    body::Empty,
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header, http::HeaderValue, StatusCode};
use interlink::service::Service;
use log::error;
use serde::Serialize;
use tokio::io::split;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    blaze::{packet::PacketCodec, session::Session},
    database::entity::User,
    http::{
        middleware::upgrade::BlazeUpgrade,
        models::{
            client::{AuthRequest, AuthResponse},
            HttpError, HttpResult,
        },
    },
    services::tokens::Tokens,
    state::{App, VERSION},
    utils::hashing::{hash_password, verify_password},
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
        version: VERSION,
    })
}

/// POST /ark/client/login
pub async fn login(Json(req): Json<AuthRequest>) -> HttpResult<AuthResponse> {
    let db = App::database();
    let user = User::get_by_username(db, &req.username)
        .await?
        .ok_or(HttpError::new("Username not found", StatusCode::NOT_FOUND))?;

    if !verify_password(&req.password, &user.password) {
        return Err(HttpError::new(
            "Incorrect password",
            StatusCode::BAD_REQUEST,
        ));
    }

    let token = Tokens::service_claim(user.id);

    Ok(Json(AuthResponse { token }))
}

/// POST /ark/client/create
pub async fn create(Json(req): Json<AuthRequest>) -> HttpResult<AuthResponse> {
    let db = App::database();

    if User::get_by_username(db, &req.username).await?.is_some() {
        return Err(HttpError::new(
            "Username already taken",
            StatusCode::CONFLICT,
        ));
    }

    let password = hash_password(&req.password).map_err(|_| {
        HttpError::new("Failed to hash password", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let user = User::create_user(req.username, password, db).await?;
    let token = Tokens::service_claim(user.id);

    Ok(Json(AuthResponse { token }))
}

/// GET /ark/client/upgrade
pub async fn upgrade(upgrade: BlazeUpgrade) -> Result<Response, HttpError> {
    let db = App::database();
    let user = Tokens::service_verify(db, upgrade.host_target.token.as_ref())
        .await
        .map_err(|err| HttpError::new_owned(err.to_string(), StatusCode::BAD_REQUEST))?;

    tokio::spawn(async move {
        let socket = match upgrade.upgrade().await {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to upgrade blaze socket: {}", err);
                return;
            }
        };

        Session::create(|ctx| {
            // Attach reader and writers to the session context
            let (read, write) = split(socket.upgrade);
            let read = FramedRead::new(read, PacketCodec);
            let write = FramedWrite::new(write, PacketCodec);

            ctx.attach_stream(read, true);
            let writer = ctx.attach_sink(write);

            Session::new(writer, socket.host_target, user)
        });
    });

    let mut response = Empty::new().into_response();
    // Use the switching protocols status code
    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

    let headers = response.headers_mut();
    // Add the upgraidng headers
    headers.insert(header::CONNECTION, HeaderValue::from_static("upgrade"));
    headers.insert(header::UPGRADE, HeaderValue::from_static("blaze"));

    Ok(response)
}
