//! This module contains HTTP routes and logic used between the server
//! and the PocketArk client

use crate::{
    blaze::{router::BlazeRouter, session::Session},
    database::entity::{
        users::{CreateUser, UserId},
        Currency, SharedData, StrikeTeam, User,
    },
    definitions::items::create_default_items,
    http::{
        middleware::{json_validated::JsonValidated, upgrade::BlazeUpgrade},
        models::{
            client::{
                ClientError, CreateUserRequest, LoginUserRequest, ServerDetailsResponse,
                TokenResponse,
            },
            DynHttpError, HttpResult,
        },
    },
    services::sessions::{Sessions, VerifyError},
    utils::hashing::{hash_password, verify_password},
    VERSION,
};
use anyhow::Context;
use axum::{response::IntoResponse, Extension, Json};
use hyper::{header, http::HeaderValue, StatusCode};
use log::error;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

/// GET /ark/client/details
///
/// Used by clients to get details about the server before
/// it connects
pub async fn details() -> Json<ServerDetailsResponse> {
    Json(ServerDetailsResponse {
        ident: "POCKET_ARK_SERVER",
        version: VERSION,
    })
}

/// POST /ark/client/login
///
/// Used by the client tool to login to an account on the server
pub async fn login(
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,
    JsonValidated(LoginUserRequest { email, password }): JsonValidated<LoginUserRequest>,
) -> HttpResult<TokenResponse> {
    // Find the user requested
    let user = User::by_email(&db, &email)
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
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,
    JsonValidated(CreateUserRequest {
        email,
        username,
        password,
    }): JsonValidated<CreateUserRequest>,
) -> HttpResult<TokenResponse> {
    // Ensure the email doesn't exist already
    if User::email_exists(&db, &email).await? {
        return Err(ClientError::EmailTaken.into());
    }

    // Ensure the username doesn't exist already
    if User::username_exists(&db, &username).await? {
        return Err(ClientError::UsernameAlreadyTaken.into());
    }

    let password = hash_password(&password).context("Failed to hash password")?;

    let create = CreateUser {
        email,
        username,
        password,
    };

    let user = db
        .transaction(|db| {
            Box::pin(async move {
                // Create the user account
                let user = User::create(db, create).await?;

                // Give the user all the default items
                create_default_items(db, &user).await?;

                // Give the user the default currencies
                Currency::set_default(db, &user).await?;

                // Setup the user shared data
                SharedData::create_default(db, &user).await?;

                // Setup the user strike teams
                StrikeTeam::create_default(db, &user).await?;

                Ok::<_, DynHttpError>(user)
            })
        })
        .await?;

    let token = sessions.create_token(user.id);

    Ok(Json(TokenResponse { token }))
}

/// GET /ark/client/upgrade
pub async fn upgrade(
    Extension(router): Extension<Arc<BlazeRouter>>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(sessions): Extension<Arc<Sessions>>,

    upgrade: BlazeUpgrade,
) -> Result<impl IntoResponse, DynHttpError> {
    let user_id: UserId = sessions.verify_token(&upgrade.token)?;

    let user = User::by_id(&db, user_id)
        .await?
        .ok_or(VerifyError::Invalid)?;

    // Handle the client upgrading in a new task
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

    // Tell the client to switch protocols
    Ok((
        StatusCode::SWITCHING_PROTOCOLS,
        [
            (header::CONNECTION, HeaderValue::from_static("upgrade")),
            (header::UPGRADE, HeaderValue::from_static("blaze")),
        ],
    ))
}
