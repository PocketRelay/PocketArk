use super::HttpError;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Username not found")]
    InvalidUsername,

    #[error("Incorrect password")]
    IncorrectPassword,

    /// Username is already taken
    #[error("Email already in use")]
    EmailTaken,

    /// Username is already taken
    #[error("Username already in use")]
    UsernameAlreadyTaken,

    /// Failed to hash a password when creating an account
    #[error("Server error")]
    FailedHashPassword,

    #[error("Auth failed")]
    AuthFailed,
}

impl HttpError for ClientError {
    fn status(&self) -> StatusCode {
        match self {
            ClientError::InvalidUsername => StatusCode::NOT_FOUND,
            ClientError::IncorrectPassword | ClientError::AuthFailed => StatusCode::BAD_REQUEST,
            ClientError::UsernameAlreadyTaken | ClientError::EmailTaken => StatusCode::CONFLICT,
            ClientError::FailedHashPassword => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Request to create a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// The email for the user
    pub email: String,
    /// The username for the user
    pub username: String,
    /// The password for the user
    pub password: String,
}

/// Request to login to a user
#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    /// The user email
    pub email: String,
    /// The user password
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}
