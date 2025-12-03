use log::info;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database as SeaDatabase;
use std::{
    fs::{File, create_dir_all},
    path::Path,
};

pub mod entity;
mod migration;
/// Testing seeding logic
#[cfg(test)]
mod seed;

// Re-exports of database types
pub use sea_orm::DatabaseConnection;
pub use sea_orm::DbErr;

/// Database error result type
pub type DbResult<T> = Result<T, DbErr>;

const DATABASE_PATH: &str = "data/app.db";
const DATABASE_PATH_URL: &str = "sqlite:data/app.db";

pub async fn init() -> DatabaseConnection {
    info!("Connected to database..");
    connect_database().await
}

/// Connects to the database
async fn connect_database() -> DatabaseConnection {
    let path = Path::new(&DATABASE_PATH);

    // Create path to database file if missing
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        create_dir_all(parent).expect("Unable to create parent directory for sqlite database");
    }

    // Create the database if file is missing
    if !path.exists() {
        File::create(path).expect("Unable to create sqlite database file");
    }

    // Connect to database
    let connection = SeaDatabase::connect(DATABASE_PATH_URL)
        .await
        .expect("Unable to create database connection");

    // Run migrations
    Migrator::up(&connection, None)
        .await
        .expect("Unable to run database migrations");

    connection
}
