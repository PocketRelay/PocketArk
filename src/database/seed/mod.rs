use super::{
    connect_database,
    entity::{InventoryItem, User, currency::CurrencyType},
};
use crate::{
    database::entity::{
        Character, Currency, SharedData, shared_data::SharedProgression, users::CreateUser,
    },
    definitions::{
        classes::{Classes, PointMap},
        i18n::{I18nDescription, I18nName},
        items::Items,
        level_tables::{LevelTables, ProgressionXp},
    },
    utils::{hashing::hash_password, logging::setup_test_logging},
};

#[tokio::test]
#[ignore]
pub async fn seed() {
    setup_test_logging();

    let db = connect_database().await;

    let create_user = CreateUser {
        email: "test@test.com".to_string(),
        username: "Test".to_string(),
        password: hash_password("test").unwrap(),
    };

    let user = User::create(&db, create_user).await.unwrap();

    let item_definitions = Items::get();
    let classes = Classes::get();
    let level_tables = LevelTables::get();

    // Initialize the users data
    // InventoryItem::create_default(&db, &user, &items, &characters)
    //     .await
    //     .unwrap();
    Currency::add_many(
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

    let mut shared_data = SharedData::create_default(&db, &user).await.unwrap();
    // StrikeTeam::create_default(&db, &user).await.unwrap();

    // Insert the initial prestige data if we don't have any
    // (Needs to happen *before* append_prestige_before to ensure it shows up in the "before" state)

    // All all the items
    for definition in item_definitions.all() {
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
            .by_name(&class.level_name)
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

        let prestige_level_table = level_tables
            .by_name(&class.prestige_level_name)
            .expect("Missing prestige level table");

        if !shared_data
            .shared_progression
            .0
            .iter()
            .any(|value| value.name == class.prestige_level_name)
        {
            shared_data.shared_progression.0.push(SharedProgression {
                i18n_name: I18nName::raw(""),
                i18n_description: I18nDescription::raw(""),
                level: 0,
                name: class.prestige_level_name,
                xp: prestige_level_table.initial_progression(),
            });
            shared_data = shared_data.save_progression(&db).await.unwrap();
        }
    }
}
