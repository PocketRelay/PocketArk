use crate::dto::users::UserId;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct SeenArticleDto {
    pub id: i64,
    pub user_id: UserId,
    pub article_id: Uuid,
}
