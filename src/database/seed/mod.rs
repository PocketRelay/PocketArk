use chrono::Local;
use rand::{distributions::Uniform, Rng};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
};
use std::fmt::Write;
use tokio::{task::JoinSet, try_join};

use crate::{
    services::{
        character::CharacterService,
        items::{BaseCategory, Category, ItemsService},
    },
    utils::logging::setup_test_logging,
};

use super::{
    connect_database,
    entity::{Character, InventoryItem, User},
};

#[tokio::test]
#[ignore]
pub async fn seed() {
    setup_test_logging();

    let db = connect_database().await;

    let user = User::get_user(&db, 1).await.unwrap().unwrap();

    let items = ItemsService::new();
    let characters = CharacterService::new();

    for definition in items.items.all() {
        let item = InventoryItem::add_item(
            &db,
            &user,
            definition.name,
            definition.capacity.unwrap_or(100_000),
            definition.capacity,
        )
        .await
        .unwrap();

        // Handle character creation if the item is a character item
        if definition
            .category
            .is_within(&Category::Base(BaseCategory::Characters))
        {
            Character::create_from_item(&db, &characters, &user, &definition.name)
                .await
                .unwrap();
        }
    }
}
