use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::dto::users::UserId;

#[derive(Debug, FromRow)]
pub struct SeenArticleDto {
    pub id: i64,
    pub user_id: UserId,
    pub article_id: Uuid,
}
