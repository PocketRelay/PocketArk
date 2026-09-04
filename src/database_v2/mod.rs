pub mod dto;
pub mod extensions;
pub mod migrations;
pub mod repositories;

use std::{fs::create_dir_all, path::Path};

pub use sqlx::SqliteExecutor as DbExecutor;
use sqlx::{ConnectOptions, SqlitePool, sqlite::SqliteConnectOptions};

use migrations::{apply_migrations, initialize_migrations_table};

pub type DbErr = sqlx::Error;
pub type DbResult<T> = Result<T, sqlx::Error>;

const DATABASE_PATH: &str = "data/app.db";

pub async fn connect_database() -> DbResult<SqlitePool> {
    let path = Path::new(&DATABASE_PATH);

    // Create path to database file if missing
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        create_dir_all(parent).expect("Unable to create parent directory for sqlite database");
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true)
        .to_url_lossy();

    let pool = SqlitePool::connect(options.as_str()).await?;
    initialize_migrations_table(&pool).await?;

    {
        let mut transaction = pool.begin().await?;
        apply_migrations(&mut transaction).await?;
        transaction.commit().await?;
    }

    Ok(pool)
}
