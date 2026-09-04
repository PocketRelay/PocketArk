use std::collections::HashSet;

use crate::database_v2::{DbExecutor, DbResult, dto::migration::CreateMigrationDto};

/// Repository for accessing the table for applied database migrations
pub struct MigrationRepository;

impl MigrationRepository {
    /// Create a new migration
    pub async fn create(db: impl DbExecutor<'_>, create: CreateMigrationDto) -> DbResult<()> {
        sqlx::query(
            r#"
            INSERT INTO "pocket_ark_migrations" ("name", "applied_at")
            VALUES (?, ?)
        "#,
        )
        .bind(create.name)
        .bind(create.applied_at)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Find the names of all the applied migrations
    pub async fn all_applied_names(db: impl DbExecutor<'_>) -> DbResult<HashSet<String>> {
        sqlx::query_scalar(r#"SELECT "name" FROM "pocket_ark_migrations""#)
            .fetch_all(db)
            .await
            .map(|results: Vec<String>| HashSet::from_iter(results))
    }
}
