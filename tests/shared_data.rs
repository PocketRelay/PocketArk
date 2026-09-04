use pocket_ark::database_v2::{
    dto::{
        shared_data::{CreateSharedDataDto, SharedDataDto},
        users::UserDto,
    },
    repositories::shared_data::SharedDataRepository,
};
use sqlx::SqlitePool;

use crate::helpers::{mock_user, test_database};

pub mod helpers;

async fn mock_create_shared_data(db: &SqlitePool, user: &UserDto) -> SharedDataDto {
    SharedDataRepository::create(
        db,
        CreateSharedDataDto {
            user_id: user.id,
            shared_equipment: serde_json::json!({ "equipment": true }),
            shared_progression: serde_json::json!({ "progression": true }),
            shared_stats: serde_json::json!({"stats": true }),
        },
    )
    .await
    .unwrap()
}

/// Tests that initially the user should have no shared data
#[tokio::test]
async fn test_user_initial_shared_data() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let shared_data = SharedDataRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert!(shared_data.is_none());
}

/// Tests that initially the user should have no shared data
#[tokio::test]
async fn test_create_user_shared_data() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let shared_data = SharedDataRepository::create(
        &db,
        CreateSharedDataDto {
            user_id: user.id,
            shared_equipment: serde_json::json!({ "equipment": true }),
            shared_progression: serde_json::json!({ "progression": true }),
            shared_stats: serde_json::json!({"stats": true }),
        },
    )
    .await
    .unwrap();

    assert_eq!(shared_data.user_id, user.id);
    assert_eq!(
        shared_data.shared_equipment,
        serde_json::json!({  "equipment": true })
    );
    assert_eq!(
        shared_data.shared_progression,
        serde_json::json!({  "progression": true })
    );
    assert_eq!(
        shared_data.shared_stats,
        serde_json::json!({  "stats": true })
    );
}

/// Tests that we can retrieve the users shared data
#[tokio::test]
async fn test_get_user_shared_data() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let shared_data = mock_create_shared_data(&db, &user).await;
    let user_shared_data = SharedDataRepository::get_by_user(&db, user.id)
        .await
        .unwrap()
        .expect("user shared data should exist");
    assert_eq!(shared_data, user_shared_data);
}

/// Tests that we can update the users active character
#[tokio::test]
async fn test_set_user_active_character() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let shared_data = mock_create_shared_data(&db, &user).await;
    assert_eq!(shared_data.active_character_id, None);

    let new_active_character = uuid::uuid!("92d879a2-2b6e-4091-9305-1fb1943df014");

    SharedDataRepository::set_user_active_character(&db, user.id, new_active_character)
        .await
        .unwrap();

    let user_shared_data = SharedDataRepository::get_by_user(&db, user.id)
        .await
        .unwrap()
        .expect("user shared data should exist");
    assert_eq!(
        user_shared_data.active_character_id,
        Some(new_active_character)
    );
}

/// Tests that we can update the users shared progression
#[tokio::test]
async fn test_set_user_shared_progression() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let shared_data = mock_create_shared_data(&db, &user).await;
    assert_eq!(shared_data.active_character_id, None);

    let new_progression = serde_json::json!({"new_progression": true});

    SharedDataRepository::set_user_shared_progression(&db, user.id, new_progression.clone())
        .await
        .unwrap();

    let user_shared_data = SharedDataRepository::get_by_user(&db, user.id)
        .await
        .unwrap()
        .expect("user shared data should exist");
    assert_eq!(user_shared_data.shared_progression, new_progression);
}

/// Tests that we can update the users shared equipment
#[tokio::test]
async fn test_set_user_shared_equipment() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let shared_data = mock_create_shared_data(&db, &user).await;
    assert_eq!(shared_data.active_character_id, None);

    let new_equipment = serde_json::json!({"new_equipment": true});

    SharedDataRepository::set_user_shared_equipment(&db, user.id, new_equipment.clone())
        .await
        .unwrap();

    let user_shared_data = SharedDataRepository::get_by_user(&db, user.id)
        .await
        .unwrap()
        .expect("user shared data should exist");
    assert_eq!(user_shared_data.shared_equipment, new_equipment);
}
