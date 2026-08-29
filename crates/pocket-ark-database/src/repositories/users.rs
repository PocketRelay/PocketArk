use crate::dto::users::{CreateUserDto, NormalizedEmail, UserDto, UserId};
use crate::{DbExecutor, DbResult};

pub struct UserRepository;

impl UserRepository {
    /// Create a new user
    pub async fn create(db: impl DbExecutor<'_>, create: CreateUserDto) -> DbResult<UserDto> {
        sqlx::query_as(
            r#"
            INSERT INTO "users" ("email", "username", "password")
            VALUES (?, ?, ?)
            RETURNING *
        "#,
        )
        .bind(create.email.into_inner())
        .bind(create.username)
        .bind(create.password)
        .fetch_one(db)
        .await
    }

    /// Check if any users exist that have the matching usernames
    pub async fn is_username_taken(db: impl DbExecutor<'_>, username: &str) -> DbResult<bool> {
        sqlx::query(r#"SELECT 1 FROM "users" WHERE "username" = ?"#)
            .bind(username)
            .fetch_optional(db)
            .await
            .map(|value| value.is_some())
    }

    /// Checks if any users exist that have the matching email
    pub async fn is_email_taken(
        db: impl DbExecutor<'_>,
        email: &NormalizedEmail,
    ) -> DbResult<bool> {
        sqlx::query(r#"SELECT 1 FROM "users" WHERE "email" = ?"#)
            .bind(email.as_str())
            .fetch_optional(db)
            .await
            .map(|value| value.is_some())
    }

    /// Get a user by ID
    pub async fn get_by_id(db: impl DbExecutor<'_>, id: UserId) -> DbResult<Option<UserDto>> {
        sqlx::query_as(r#"SELECT * FROM "users" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(db)
            .await
    }

    /// Get a user by email
    pub async fn get_by_email(
        db: impl DbExecutor<'_>,
        email: &NormalizedEmail,
    ) -> DbResult<Option<UserDto>> {
        sqlx::query_as(r#"SELECT * FROM "users" WHERE "email" = ?"#)
            .bind(email.as_str())
            .fetch_optional(db)
            .await
    }
}
