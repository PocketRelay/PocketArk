use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::models::Sku;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    pub auth_method: String,
    pub language: String,
    pub auth_code: String,
    pub auth_token: String,
    pub email_address: String,
    pub password: String,
    pub persona_id: u32,
    pub sku: Sku,
    pub metadata_tag: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub session_id: String,
    pub user: AuthUser,
    pub pid: String,
    pub server_time: DateTime<Utc>,
    pub language: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUser {
    pub roles: &'static [&'static str],
    #[serde(rename = "pid")]
    pub pid: u32,
    pub persona_id: u32,
    pub sku: Sku,
    pub anonymous: bool,
    pub name: String,
}
