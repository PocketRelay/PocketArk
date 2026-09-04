use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

use crate::database_v2::dto::users::UserId;

pub type ItemId = i64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
pub enum InventoryItemEarnedBy {
    #[serde(rename = "granted")]
    #[sqlx(rename = "granted")]
    Granted,
}

#[derive(Debug, FromRow, PartialEq, Eq, Clone)]
pub struct InventoryItemDto {
    pub id: ItemId,
    pub item_id: Uuid,
    pub user_id: UserId,
    pub definition_name: Uuid,
    pub stack_size: u32,
    pub seen: bool,
    #[sqlx(json)]
    pub instance_attributes: serde_json::Map<String, serde_json::Value>,
    pub created: DateTime<Utc>,
    pub last_grant: DateTime<Utc>,
    pub earned_by: InventoryItemEarnedBy,
    pub restricted: bool,
}

#[derive(Debug, Clone)]
pub struct CreateInventoryItemDto {
    pub user_id: UserId,
    pub definition_name: Uuid,
    pub stack_size: u32,
    pub capacity: Option<u32>,
    pub created_at: DateTime<Utc>,
}
