//! This module contains HTTP routes and logic used between the server
//! and the PocketArk client

use std::sync::Arc;

use crate::{
    blaze::{router::BlazeRouter, session::Session},
    database::entity::{Currency, InventoryItem, SharedData, StrikeTeam, User},
    http::{
        middleware::upgrade::BlazeUpgrade,
        models::{
            client::{AuthRequest, AuthResponse},
            HttpResult, RawHttpError,
        },
    },
    services::sessions::{Sessions, VerifyError},
    state::{App, VERSION},
    utils::hashing::{hash_password, verify_password},
};
use axum::{
    body::Empty,
    response::{IntoResponse, Response},
    Extension, Json,
};
use hyper::{header, http::HeaderValue, StatusCode};
use log::error;
use sea_orm::DatabaseConnection;
use serde::Serialize;

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
pub async fn login(
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,
    Json(req): Json<AuthRequest>,
) -> HttpResult<AuthResponse> {
    let user = User::get_by_username(&db, &req.username)
        .await?
        .ok_or(RawHttpError::new(
            "Username not found",
            StatusCode::NOT_FOUND,
        ))?;

    if !verify_password(&req.password, &user.password) {
        return Err(RawHttpError::new(
            "Incorrect password",
            StatusCode::BAD_REQUEST,
        ));
    }

    let token = sessions.create_token(user.id);

    Ok(Json(AuthResponse { token }))
}

/// POST /ark/client/create
pub async fn create(
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,
    Json(req): Json<AuthRequest>,
) -> HttpResult<AuthResponse> {
    if User::get_by_username(&db, &req.username).await?.is_some() {
        return Err(RawHttpError::new(
            "Username already taken",
            StatusCode::CONFLICT,
        ));
    }

    let password = hash_password(&req.password).map_err(|_| {
        RawHttpError::new("Failed to hash password", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let user = User::create_user(&db, req.username, password).await?;

    let services = App::services();

    // Initialize the users data
    InventoryItem::create_default(&db, &user, &services.items, &services.character).await?;
    Currency::set_default(&db, &user).await?;
    SharedData::create_default(&db, &user).await?;
    StrikeTeam::create_default(&db, &user).await?;

    let token = sessions.create_token(user.id);

    Ok(Json(AuthResponse { token }))
}

/// GET /ark/client/upgrade
pub async fn upgrade(
    Extension(router): Extension<Arc<BlazeRouter>>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,

    upgrade: BlazeUpgrade,
) -> Result<Response, RawHttpError> {
    let user_id: u32 = sessions
        .verify_token(&upgrade.token)
        .map_err(|err| RawHttpError::new("Auth failed", StatusCode::BAD_REQUEST))?;

    let user = User::get_user(&db, user_id)
        .await?
        .ok_or(VerifyError::Invalid)
        .map_err(|err| RawHttpError::new("Auth failed", StatusCode::BAD_REQUEST))?;

    tokio::spawn(async move {
        let socket = match upgrade.upgrade().await {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to upgrade blaze socket: {}", err);
                return;
            }
        };

        Session::start(socket.upgrade, user, router, sessions).await;
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
