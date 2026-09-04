use crate::database_v2::{
    DbResult, dto::migration::CreateMigrationDto, repositories::migration::MigrationRepository,
};
use sqlx::{SqlStr, SqliteExecutor, SqliteTransaction, types::chrono::Utc};
use std::ops::DerefMut;

pub const MIGRATIONS: &[(&str, &str)] = &[
    (
        "m20230714_105755_create_users",
        include_str!("./m20230714_105755_create_users.sql"),
    ),
    (
        "m20230714_105927_create_currency",
        include_str!("./m20230714_105927_create_currency.sql"),
    ),
    (
        "m20230714_105933_create_inventory_item",
        include_str!("./m20230714_105933_create_inventory_item.sql"),
    ),
    (
        "m20230714_105940_create_seen_articles",
        include_str!("./m20230714_105940_create_seen_articles.sql"),
    ),
    (
        "m20230714_105946_create_characters",
        include_str!("./m20230714_105946_create_characters.sql"),
    ),
    (
        "m20230714_112535_create_shared_data",
        include_str!("./m20230714_112535_create_shared_data.sql"),
    ),
    (
        "m20230720_145347_create_challenge_progress",
        include_str!("./m20230720_145347_create_challenge_progress.sql"),
    ),
    (
        "m20230731_123814_create_strike_teams",
        include_str!("./m20230731_123814_create_strike_teams.sql"),
    ),
    (
        "m20231223_184934_create_strike_team_missions",
        include_str!("./m20231223_184934_create_strike_team_missions.sql"),
    ),
    (
        "m20231223_185554_create_strike_team_mission_progress",
        include_str!("./m20231223_185554_create_strike_team_mission_progress.sql"),
    ),
];

/// Initializes the root migrations table in preparation for applying migrations
pub async fn initialize_migrations_table(db: impl SqliteExecutor<'_>) -> DbResult<()> {
    let migration = include_str!("./m0_create_migrations.sql");

    sqlx::raw_sql(SqlStr::from_static(migration))
        .execute(db)
        .await?;

    Ok(())
}

pub async fn apply_migrations(db: &mut SqliteTransaction<'_>) -> DbResult<()> {
    let applied_migration_names = MigrationRepository::all_applied_names(db.deref_mut()).await?;

    let new_migrations = MIGRATIONS
        .iter()
        // Skip any migrations we already applied
        .filter(|(migration_name, _)| !applied_migration_names.contains(*migration_name));

    for (migration_name, migration) in new_migrations {
        // Apply the migration
        apply_migration_sql(db, migration_name, migration).await?;

        // Store the applied migration
        MigrationRepository::create(
            db.deref_mut(),
            CreateMigrationDto {
                name: migration_name.to_string(),
                applied_at: Utc::now(),
            },
        )
        .await?;
    }

    Ok(())
}

async fn apply_migration_sql(
    db: &mut SqliteTransaction<'_>,
    migration_name: &str,
    migration: &'static str,
) -> DbResult<()> {
    // Split the SQL queries into multiple queries
    let queries = migration
        .split(';')
        .map(|query| query.trim())
        .filter(|query| !query.is_empty());

    for query in queries {
        let result = sqlx::query(SqlStr::from_static(query))
            .execute(db.deref_mut())
            .await
            .inspect_err(|error| {
                eprintln!("{error:?} {query}");
                tracing::error!(?error, ?migration_name, "failed to perform migration")
            })?;
        let rows_affected = result.rows_affected();
        tracing::debug!(?migration_name, ?rows_affected, "applied migration query");
    }

    Ok(())
}
