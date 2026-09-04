use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::dto::users::UserId;

#[derive(Debug, FromRow, Clone, PartialEq, Eq)]
pub struct SharedDataDto {
    pub user_id: UserId,

    pub active_character_id: Option<Uuid>,

    #[sqlx(json)]
    pub shared_stats: serde_json::Value,
    #[sqlx(json)]
    pub shared_equipment: serde_json::Value,
    #[sqlx(json)]
    pub shared_progression: serde_json::Value,
}

#[derive(Debug)]
pub struct CreateSharedDataDto {
    pub user_id: UserId,
    pub shared_stats: serde_json::Value,
    pub shared_equipment: serde_json::Value,
    pub shared_progression: serde_json::Value,
}
