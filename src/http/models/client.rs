use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthenticateResponse {
    pub token: String,
}
