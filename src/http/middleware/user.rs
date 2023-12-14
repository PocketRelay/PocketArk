use crate::{
    database::entity::User,
    http::models::HttpError,
    services::sessions::{Sessions, VerifyError},
};
use axum::extract::FromRequestParts;
use futures::future::BoxFuture;
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct Auth(pub User);

/// The HTTP header that contains the authentication token
const TOKEN_HEADER: &str = "X-Token";

impl<S> FromRequestParts<S> for Auth {
    type Rejection = HttpError;

    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut axum::http::request::Parts,
        _state: &'b S,
    ) -> BoxFuture<'c, Result<Self, Self::Rejection>>
    where
        'a: 'c,
        'b: 'c,
        Self: 'c,
    {
        let db = parts
            .extensions
            .get::<DatabaseConnection>()
            .expect("Database connection extension missing")
            .clone();

        let sessions: Arc<Sessions> = parts
            .extensions
            .get::<Arc<Sessions>>()
            .expect("Sessions extension missing")
            .clone();

        Box::pin(async move {
            // Extract the token from the headers
            let token = parts
                .headers
                .get(TOKEN_HEADER)
                .and_then(|value| value.to_str().ok())
                .ok_or(HttpError::new("Missing session", StatusCode::BAD_REQUEST))?;

            let user_id: u32 = sessions
                .verify_token(token)
                .map_err(|_| HttpError::new("Auth failed", StatusCode::INTERNAL_SERVER_ERROR))?;

            let user = User::get_user(&db, user_id)
                .await?
                .ok_or(VerifyError::Invalid)
                .map_err(|_| HttpError::new("Auth failed", StatusCode::INTERNAL_SERVER_ERROR))?;

            Ok(Self(user))
        })
    }
}
