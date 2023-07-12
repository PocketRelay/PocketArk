use chrono::{DateTime, Utc};
use serde::{
    de::{DeserializeOwned, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    pub auth_method: String,
    pub language: String,
    pub auth_code: String,
    pub auth_token: String,
    pub email_address: String,
    pub password: String,
    pub persona_id: u64,
    pub sku: Sku,
    pub metadata_tag: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub session_id: Uuid,
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
    pub pid: u64,
    pub persona_id: u64,
    pub sku: Sku,
    pub anonymous: bool,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sku {
    pub title: String,
    pub platform: String,
}
