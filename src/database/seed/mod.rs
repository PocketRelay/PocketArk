use super::{
    connect_database,
    entity::{currency::CurrencyType, InventoryItem, User},
};
use crate::{
    database::entity::{Character, Currency, SharedData},
    services::{
        character::{
            class::{ClassDefinitions, PointMap},
            levels::{LevelTables, ProgressionXp},
        },
        items::ItemsService,
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
    let classes = ClassDefinitions::new().unwrap();
    let level_tables = LevelTables::new().unwrap();

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

    // All all the items
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
    }

    // Add all the characters
    for class in classes.all() {
        let level = 20;
        // Get the current xp progression values
        let xp: ProgressionXp = level_tables
            .get(&class.level_name)
            .unwrap()
            .get_xp_values(level)
            .unwrap()
            .into();

        let points: PointMap = PointMap {
            skill_points: Some(255),
        };
        let skill_trees = class.skill_trees.clone();
        let attributes = class.attributes.clone();
        let bonus = class.bonus.clone();
        let equipment = class.default_equipments.clone();
        let customization = class.default_customization.clone();

        Character::create(
            &db,
            &user,
            class.name,
            level,
            xp,
            points,
            skill_trees,
            attributes,
            bonus,
            equipment,
            customization,
        )
        .await
        .unwrap();
    }
}
