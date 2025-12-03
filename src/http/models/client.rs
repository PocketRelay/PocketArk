use super::HttpError;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Account not found")]
    AccountNotFound,

    #[error("Incorrect password")]
    IncorrectPassword,

    /// Username is already taken
    #[error("Email already in use")]
    EmailTaken,

    /// Username is already taken
    #[error("Username already in use")]
    UsernameAlreadyTaken,
}

impl HttpError for ClientError {
    fn status(&self) -> StatusCode {
        match self {
            ClientError::AccountNotFound => StatusCode::NOT_FOUND,
            ClientError::IncorrectPassword => StatusCode::BAD_REQUEST,
            ClientError::UsernameAlreadyTaken | ClientError::EmailTaken => StatusCode::CONFLICT,
        }
    }
}

/// Response containing details about the server
#[derive(Serialize)]
pub struct ServerDetailsResponse {
    /// Identifier used to ensure the server is a Pocket Ark server
    pub ident: &'static str,
    /// The server version
    pub version: &'static str,
    /// Random association token for the client to use
    pub association: String,
    /// Port the tunnel server is running on
    pub tunnel_port: Option<u16>,
}

/// Request to create a new user
#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    /// The email for the user
    #[validate(email)]
    pub email: String,
    /// The username for the user
    #[validate(length(min = 4, max = 16))]
    pub username: String,
    /// The password for the user
    #[validate(length(min = 1))]
    pub password: String,
}

/// Request to login to a user
#[derive(Debug, Validate, Deserialize)]
pub struct LoginUserRequest {
    /// The user email
    #[validate(email)]
    pub email: String,
    /// The user password
    #[validate(length(min = 1))]
    pub password: String,
}

/// Response JSON containing a token
#[derive(Serialize)]
pub struct TokenResponse {
    /// The token field
    pub token: String,
}
