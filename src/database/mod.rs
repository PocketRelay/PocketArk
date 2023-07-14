use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveValue, Database as SeaDatabase, EntityTrait, IntoActiveModel};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};

pub mod entity;
mod migration;

// Re-exports of database types
pub use sea_orm::DatabaseConnection;
pub use sea_orm::DbErr;

use crate::database::entity::{inventory_items::ActiveModel, InventoryItem};

/// Database error result type
pub type DbResult<T> = Result<T, DbErr>;

const DATABASE_PATH: &str = "data/app.db";
const DATABASE_PATH_URL: &str = "sqlite:data/app.db";

pub async fn init() -> DatabaseConnection {
    info!("Connected to database..");

    let path = Path::new(&DATABASE_PATH);

    // Create path to database file if missing
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_dir_all(parent).expect("Unable to create parent directory for sqlite database");
        }
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
    fill_items(&connection).await;
    connection
}

// pub async fn fill_items(c: &DatabaseConnection) {
//     static PLACEHOLDER_INVENTORY: &str =
//         include_str!("../resources/data/placeholderInventory.json");
//     let mut items: Vec<InventoryItem> = serde_json::from_str(PLACEHOLDER_INVENTORY).unwrap();
//     items.iter_mut().for_each(|item| {
//         item.user_id = 1;
//     });
//     let items = items
//         .into_iter()
//         .map(|value| value.into_active_model())
//         .map(|value| ActiveModel {
//             id: ActiveValue::NotSet,
//             ..value
//         });
//     entity::inventory_items::Entity::insert_many(items)
//         .exec(c)
//         .await
//         .unwrap();
//     debug!("Inserted all player inventory data")
// }

// pub async fn fill_characters(c: &DatabaseConnection) {
//     static PLACEHOLDER_INVENTORY: &str =
//         include_str!("../resources/data/placeholderInventory.json");
//     let mut items: Vec<InventoryItem> = serde_json::from_str(PLACEHOLDER_INVENTORY).unwrap();
//     items.iter_mut().for_each(|item| {
//         item.user_id = 1;
//     });
//     let items = items
//         .into_iter()
//         .map(|value| value.into_active_model())
//         .map(|value| ActiveModel {
//             id: ActiveValue::NotSet,
//             ..value
//         });
//     entity::inventory_items::Entity::insert_many(items)
//         .exec(c)
//         .await
//         .unwrap();
//     debug!("Inserted all player inventory data")
// }
