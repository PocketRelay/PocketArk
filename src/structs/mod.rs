use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub user: AuthUser,
    pub pid: String,
    #[serde(rename = "serverTime")]
    pub server_time: DateTime<Utc>,
    pub language: String,
}

#[derive(Debug, Serialize)]
pub struct AuthUser {
    pub roles: Vec<String>,
    #[serde(rename = "pid")]
    pub player_id: u64,
    #[serde(rename = "personaId")]
    pub persona_id: u64,
    pub sku: Sku,
    pub anonymous: bool,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Sku {
    pub title: String,
    pub platform: String,
}
