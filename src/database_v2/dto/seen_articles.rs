use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database_v2::dto::users::UserId;

#[derive(Debug, FromRow)]
pub struct SeenArticleDto {
    pub id: i64,
    pub user_id: UserId,
    pub article_id: Uuid,
}
