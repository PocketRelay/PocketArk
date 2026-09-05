use chrono::Utc;
use uuid::Uuid;

use crate::{
    database_v2::{
        connect_database,
        dto::{
            character::{CreateCharacterDto, PlayStats},
            currency::{CurrencyType, CurrencyUpdateDto},
            inventory_items::CreateInventoryItemDto,
            shared_data::SharedProgression,
            users::{CreateUserDto, NormalizedEmail},
        },
        repositories::{
            characters::CharactersRepository, currency::CurrencyRepository,
            inventory_items::InventoryItemsRepository, shared_data::SharedDataRepository,
            users::UserRepository,
        },
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

    let db = connect_database().await.unwrap();

    let create_user = CreateUserDto {
        email: NormalizedEmail::new("test@test.com"),
        username: "Test".to_string(),
        password: hash_password("test").unwrap(),
    };

    let user = UserRepository::create(&db, create_user).await.unwrap();

    let item_definitions = Items::get();
    let classes = Classes::get();
    let level_tables = LevelTables::get();

    // Initialize the users data
    // InventoryItem::create_default(&db, &user, &items, &characters)
    //     .await
    //     .unwrap();
    CurrencyRepository::apply_currency_updates(
        &db,
        user.id,
        [
            (CurrencyType::Mtx, CurrencyRepository::MAX_SAFE_CURRENCY),
            (CurrencyType::Grind, CurrencyRepository::MAX_SAFE_CURRENCY),
            (CurrencyType::Mission, CurrencyRepository::MAX_SAFE_CURRENCY),
        ]
        .into_iter()
        .map(|(ty, balance)| CurrencyUpdateDto { ty, balance })
        .collect(),
    )
    .await
    .unwrap();

    let mut shared_data = SharedDataRepository::create_default(&db, user.id)
        .await
        .unwrap();
    // StrikeTeam::create_default(&db, &user).await.unwrap();

    // Insert the initial prestige data if we don't have any
    // (Needs to happen *before* append_prestige_before to ensure it shows up in the "before" state)

    // All all the items
    for definition in item_definitions.all() {
        let _item = InventoryItemsRepository::add_item(
            &db,
            CreateInventoryItemDto {
                user_id: user.id,
                definition_name: definition.name,
                stack_size: definition.capacity.unwrap_or(100_000),
                capacity: definition.capacity,
                created_at: Utc::now(),
            },
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

        CharactersRepository::create(
            &db,
            CreateCharacterDto {
                character_id: Uuid::new_v4(),
                user_id: user.id,
                class_name: class.name,
                level,
                xp,
                promotion: 0,
                points,
                points_spent: PointMap::default_spent(),
                points_granted: PointMap::default(),
                skill_trees,
                attributes,
                bonus,
                equipments: equipment,
                customization,
                play_stats: PlayStats::default(),
            },
        )
        .await
        .unwrap();

        let prestige_level_table = level_tables
            .by_name(&class.prestige_level_name)
            .expect("Missing prestige level table");

        if !shared_data
            .shared_progression
            .iter()
            .any(|value| value.name == class.prestige_level_name)
        {
            shared_data.shared_progression.push(SharedProgression {
                i18n_name: I18nName::raw(""),
                i18n_description: I18nDescription::raw(""),
                level: 0,
                name: class.prestige_level_name,
                xp: prestige_level_table.initial_progression(),
            });

            SharedDataRepository::set_user_shared_progression(
                &db,
                user.id,
                &shared_data.shared_progression,
            )
            .await
            .unwrap();
        }
    }
}
