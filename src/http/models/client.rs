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
            ClientError::UsernameAlreadyTaken => StatusCode::CONFLICT,
            ClientError::FailedHashPassword => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}
