//! This module contains HTTP routes and logic used between the server
//! and the PocketArk client

use crate::{
    blaze::{data::SessionData, router::BlazeRouter, session::Session},
    config::{Config, VERSION},
    database::{
        DbPool,
        dto::users::{CreateUserDto, NormalizedEmail},
        repositories::{
            currency::CurrencyRepository, shared_data::SharedDataRepository, users::UserRepository,
        },
    },
    definitions::{items::create_default_items, strike_teams::create_user_strike_team},
    http::{
        middleware::{
            association::Association, ip_address::IpAddress, json_validated::JsonValidated,
            upgrade::Upgrade, user::Auth,
        },
        models::{
            DynHttpError, HttpResult,
            client::{
                ClientError, CreateUserRequest, LoginUserRequest, ServerDetailsResponse,
                TokenResponse,
            },
        },
    },
    services::{
        sessions::{AssociationId, Sessions},
        tunnel::{TunnelService, http_tunnel::HttpTunnel},
    },
    utils::hashing::{hash_password, verify_password},
};
use anyhow::Context;
use axum::{
    Extension, Json,
    response::{IntoResponse, Response},
};
use hyper::{StatusCode, header, http::HeaderValue, upgrade::OnUpgrade};
use log::{debug, error};
use std::{ops::DerefMut, sync::Arc};
use uuid::Uuid;

/// GET /ark/client/details
///
/// Used by clients to get details about the server before
/// it connects
pub async fn details(
    Extension(sessions): Extension<Arc<Sessions>>,
    Extension(config): Extension<Arc<Config>>,
) -> Json<ServerDetailsResponse> {
    let association = sessions.create_assoc_token();

    Json(ServerDetailsResponse {
        ident: "POCKET_ARK_SERVER",
        version: VERSION,
        association,
        tunnel_port: config.udp_tunnel.get_exposed_port(),
    })
}

/// POST /ark/client/login
///
/// Used by the client tool to login to an account on the server
pub async fn login(
    Extension(db): Extension<DbPool>,
    Extension(sessions): Extension<Arc<Sessions>>,
    JsonValidated(LoginUserRequest { email, password }): JsonValidated<LoginUserRequest>,
) -> HttpResult<TokenResponse> {
    let email = NormalizedEmail::new(email);

    // Find the user requested
    let user = UserRepository::get_by_email(&db, &email)
        .await?
        .ok_or(ClientError::AccountNotFound)?;

    // Ensure the passwords match
    if !verify_password(&password, &user.password) {
        return Err(ClientError::IncorrectPassword.into());
    }

    let token = sessions.create_token(user.id);

    Ok(Json(TokenResponse { token }))
}

/// POST /ark/client/create
///
/// Used by the client tool to create an account on the server
pub async fn create(
    Extension(db): Extension<DbPool>,
    Extension(sessions): Extension<Arc<Sessions>>,
    JsonValidated(CreateUserRequest {
        email,
        username,
        password,
    }): JsonValidated<CreateUserRequest>,
) -> HttpResult<TokenResponse> {
    let email = NormalizedEmail::new(email);

    // Ensure the email doesn't exist already
    if UserRepository::is_email_taken(&db, &email).await? {
        return Err(ClientError::EmailTaken.into());
    }

    // Ensure the username doesn't exist already
    if UserRepository::is_username_taken(&db, &username).await? {
        return Err(ClientError::UsernameAlreadyTaken.into());
    }

    let password = hash_password(&password).context("Failed to hash password")?;

    let create = CreateUserDto {
        email,
        username,
        password,
    };

    let user = {
        let mut transaction = db.begin().await?;
        // Create the user account
        let user = UserRepository::create(transaction.deref_mut(), create).await?;

        // Give the user all the default items
        create_default_items(&mut transaction, &user).await?;

        // Give the user the default currencies
        CurrencyRepository::add_initial_currency(transaction.deref_mut(), user.id).await?;

        // Setup the user shared data
        SharedDataRepository::create_default(transaction.deref_mut(), user.id).await?;

        // Setup the user strike teams
        create_user_strike_team(&mut transaction, &user).await?;

        transaction.commit().await?;

        user
    };

    let token = sessions.create_token(user.id);

    Ok(Json(TokenResponse { token }))
}

/// GET /api/server/upgrade
///
/// Handles upgrading a HTTP connection to a blaze stream for game traffic
pub async fn upgrade(
    IpAddress(addr): IpAddress,
    Auth(user): Auth,
    Association(association_id): Association,
    Extension(router): Extension<Arc<BlazeRouter>>,
    Extension(sessions): Extension<Arc<Sessions>>,
    Upgrade(upgrade): Upgrade,
) -> Result<impl IntoResponse, DynHttpError> {
    // Handle the client upgrading in a new task
    tokio::spawn(async move {
        let io = match upgrade.await {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to upgrade blaze socket: {}", err);
                return;
            }
        };

        let id = Uuid::new_v4();

        debug!("Session started (SID: {id}, ASSOC: {association_id:?}, ADDR: {addr})");
        let data = SessionData::new(addr, association_id);
        let link = Session::start(id, io, data, router);

        let assoc = sessions.add_session(user, link.clone());
        if let Some(session) = link.upgrade() {
            session.data.set_auth(assoc);
        }
    });

    // Tell the client to switch protocols
    Ok((
        StatusCode::SWITCHING_PROTOCOLS,
        [
            (header::CONNECTION, HeaderValue::from_static("upgrade")),
            (header::UPGRADE, HeaderValue::from_static("blaze")),
        ],
    ))
}

/// GET /api/server/tunnel
///
/// Handles upgrading connections from the Pocket Relay Client tool
/// from HTTP over to the Blaze protocol to proxy the game traffic
/// as blaze sessions using HTTP Upgrade
pub async fn tunnel(
    Association(association_id): Association,
    Extension(tunnel_service): Extension<Arc<TunnelService>>,
    Upgrade(upgrade): Upgrade,
) -> Response {
    // Handle missing token
    let Some(association_id) = association_id else {
        return (StatusCode::BAD_REQUEST, "Missing association token").into_response();
    };

    // Spawn the upgrading process to its own task
    tokio::spawn(handle_upgrade_tunnel(
        upgrade,
        association_id,
        tunnel_service,
    ));

    // Let the client know to upgrade its connection
    (
        // Switching protocols status code
        StatusCode::SWITCHING_PROTOCOLS,
        // Headers required for upgrading
        [(header::CONNECTION, "upgrade"), (header::UPGRADE, "tunnel")],
    )
        .into_response()
}

/// Handles upgrading a connection and starting a new session
/// from the connection
pub async fn handle_upgrade_tunnel(
    upgrade: OnUpgrade,
    association: AssociationId,
    tunnel_service: Arc<TunnelService>,
) {
    let upgraded = match upgrade.await {
        Ok(upgraded) => upgraded,
        Err(err) => {
            error!("Failed to upgrade client connection: {err}");
            return;
        }
    };

    HttpTunnel::start(tunnel_service, association, upgraded);
}
