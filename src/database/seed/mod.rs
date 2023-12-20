use super::{
    connect_database,
    entity::{currency::CurrencyType, Character, InventoryItem, User},
};
use crate::{
    database::entity::{Currency, SharedData},
    services::{
        character::CharacterService,
        items::{BaseCategory, Category, ItemsService},
    },
    utils::{hashing::hash_password, logging::setup_test_logging},
};

#[tokio::test]
#[ignore]
pub async fn seed() {
    setup_test_logging();

    let db = connect_database().await;

    let user = User::create_user(&db, "test".to_string(), hash_password("test").unwrap())
        .await
        .unwrap();

    let items = ItemsService::new().unwrap();
    let characters = CharacterService::new().unwrap();

    // Initialize the users data
    // InventoryItem::create_default(&db, &user, &items, &characters)
    //     .await
    //     .unwrap();
    Currency::set_many(
        &db,
        &user,
        [
            (CurrencyType::Mtx, Currency::MAX_SAFE_CURRENCY),
            (CurrencyType::Grind, Currency::MAX_SAFE_CURRENCY),
            (CurrencyType::Mission, Currency::MAX_SAFE_CURRENCY),
        ],
    )
    .await
    .unwrap();
    SharedData::create_default(&db, &user).await.unwrap();
    // StrikeTeam::create_default(&db, &user).await.unwrap();

    for definition in items.items.all() {
        let _item = InventoryItem::add_item(
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
