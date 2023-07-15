use axum::extract::FromRequestParts;
use futures::future::BoxFuture;
use hyper::StatusCode;

use crate::{
    database::entity::User, http::models::HttpError, services::tokens::Tokens, state::App,
};

pub struct Auth(pub User);

/// The HTTP header that contains the authentication token
const TOKEN_HEADER: &str = "x-orbit-sid";

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
        Box::pin(async move {
            // Extract the token from the headers
            let token = parts
                .headers
                .get(TOKEN_HEADER)
                .and_then(|value| value.to_str().ok())
                .ok_or(HttpError::new("Missing session", StatusCode::BAD_REQUEST))?;

            // Verify the token claim
            let db = App::database();
            let user: User = Tokens::service_verify(db, token)
                .await
                .map_err(|err| HttpError::new("Auth failed", StatusCode::INTERNAL_SERVER_ERROR))?;

            Ok(Self(user))
        })
    }
}
