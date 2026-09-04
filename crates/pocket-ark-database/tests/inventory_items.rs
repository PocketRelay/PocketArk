use pocket_ark_database::{
    dto::{
        inventory_items::{CreateInventoryItemDto, InventoryItemDto, InventoryItemEarnedBy},
        users::UserDto,
    },
    repositories::inventory_items::InventoryItemsRepository,
};
use sqlx::{SqlitePool, types::chrono::Utc};
use uuid::Uuid;

use crate::helpers::{mock_user, test_database};

pub mod helpers;

async fn mock_create_item(
    db: &SqlitePool,
    user: &UserDto,
    definition_name: Uuid,
    stack_size: u32,
) -> InventoryItemDto {
    let now = Utc::now();
    InventoryItemsRepository::add_item(
        db,
        CreateInventoryItemDto {
            user_id: user.id,
            definition_name,
            stack_size,
            capacity: None,
            created_at: now,
        },
    )
    .await
    .unwrap()
}

/// Tests that initially the get_by_user query should succeed and produce no
/// items for a fresh user
#[tokio::test]
async fn test_user_items_get_by_user_empty() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let items = InventoryItemsRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert!(items.is_empty());
}

/// Tests that a item can be added to a fresh user
#[tokio::test]
async fn test_user_create_item() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let now = Utc::now();

    let item = InventoryItemsRepository::add_item(
        &db,
        CreateInventoryItemDto {
            user_id: user.id,
            definition_name,
            stack_size: 5,
            capacity: None,
            created_at: now,
        },
    )
    .await
    .unwrap();

    assert_eq!(item.id, 1);
    assert_eq!(item.user_id, user.id);
    assert_eq!(item.definition_name, definition_name);
    assert_eq!(item.stack_size, 5);
    assert!(!item.seen);
    assert!(item.instance_attributes.is_empty());
    assert_eq!(item.created, now);
    assert_eq!(item.last_grant, now);
    assert_eq!(item.earned_by, InventoryItemEarnedBy::Granted);
    assert!(!item.restricted);

    let mut items = InventoryItemsRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert_eq!(items.len(), 1);

    let found_item = items.pop().expect("item should exist");
    assert_eq!(item, found_item);
}

/// Tests that adding a item where a matching item already exists should just
/// increase the item stack size
#[tokio::test]
async fn test_user_create_item_add_stack_size() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");

    let original_item = mock_create_item(&db, &user, definition_name, 5).await;

    let mut items = InventoryItemsRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert_eq!(items.len(), 1);

    let original_found_item = items.pop().expect("item should exist");
    assert_eq!(original_item, original_found_item);
    let next_now = Utc::now();

    let item = InventoryItemsRepository::add_item(
        &db,
        CreateInventoryItemDto {
            user_id: user.id,
            definition_name,
            stack_size: 5,
            capacity: None,
            created_at: next_now,
        },
    )
    .await
    .unwrap();

    assert_eq!(item.stack_size, 10);
    assert_eq!(item.last_grant, next_now);

    // Creation date should remain unchanged
    assert_eq!(item.created, original_item.created);
}

/// Tests that a item stack size can be updated
#[tokio::test]
async fn test_user_set_item_stack_size() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let item = mock_create_item(&db, &user, definition_name, 5).await;

    assert_eq!(item.stack_size, 5);

    let updated = InventoryItemsRepository::set_item_stack_size(&db, user.id, definition_name, 10)
        .await
        .unwrap();

    assert!(updated);

    let mut items = InventoryItemsRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert_eq!(items.len(), 1);

    let found_item = items.pop().expect("item should exist");

    assert_eq!(found_item.stack_size, 10);
}

/// Tests that no update is reported if the target item for stack size change
/// is not found
#[tokio::test]
async fn test_user_set_stack_size_unknown_item() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let updated = InventoryItemsRepository::set_item_stack_size(&db, user.id, Uuid::new_v4(), 10)
        .await
        .unwrap();

    assert!(!updated);
}

/// Tests that items can be marked as seen
#[tokio::test]
async fn test_mark_item_seen() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    assert!(!item_1.seen);
    assert!(!item_2.seen);

    InventoryItemsRepository::mark_items_seen(
        &db,
        user.id,
        &[definition_1_name, definition_2_name],
    )
    .await
    .unwrap();

    let item_1 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_1.item_id)
        .await
        .unwrap()
        .expect("item should exist");
    let item_2 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_2.item_id)
        .await
        .unwrap()
        .expect("item should exist");

    assert!(item_1.seen);
    assert!(item_2.seen);
}

/// Tests that items can be marked as seen independently
#[tokio::test]
async fn test_mark_item_seen_independently() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    assert!(!item_1.seen);
    assert!(!item_2.seen);

    InventoryItemsRepository::mark_items_seen(&db, user.id, &[definition_1_name])
        .await
        .unwrap();

    let item_1 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_1.item_id)
        .await
        .unwrap()
        .expect("item should exist");
    let item_2 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_2.item_id)
        .await
        .unwrap()
        .expect("item should exist");

    assert!(item_1.seen);
    assert!(!item_2.seen);
}

/// Tests that no items are affected when a non existent / other id is used
#[tokio::test]
async fn test_mark_item_seen_none() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");
    let definition_3_name = uuid::uuid!("cc7bb417-fd57-4dc0-9477-5d88a2d986b3");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    assert!(!item_1.seen);
    assert!(!item_2.seen);

    InventoryItemsRepository::mark_items_seen(&db, user.id, &[definition_3_name])
        .await
        .unwrap();

    let item_1 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_1.item_id)
        .await
        .unwrap()
        .expect("item should exist");
    let item_2 = InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_2.item_id)
        .await
        .unwrap()
        .expect("item should exist");

    assert!(!item_1.seen);
    assert!(!item_2.seen);
}

/// Tests that all user items can be found and items from other users aren't included
#[tokio::test]
async fn test_get_items_by_user() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let user_2 = mock_user(&db, "test2@test.com", "test2").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    // Ensure items for other users aren't included
    let _item_3 = mock_create_item(&db, &user_2, definition_2_name, 5).await;

    let items = InventoryItemsRepository::get_by_user(&db, user.id)
        .await
        .unwrap();

    assert_eq!(items.len(), 2);

    let item_1_found = items
        .iter()
        .find(|item| item.id == item_1.id)
        .expect("item 1 should exist");
    let item_2_found = items
        .iter()
        .find(|item| item.id == item_2.id)
        .expect("item 1 should exist");

    assert_eq!(&item_1, item_1_found);
    assert_eq!(&item_2, item_2_found);
}

/// Tests that items can be found by ID
#[tokio::test]
async fn test_get_item_by_user_by_id_none() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let none = InventoryItemsRepository::get_by_user_by_item_id(
        &db,
        user.id,
        uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef"),
    )
    .await
    .unwrap();

    assert!(none.is_none());
}

/// Tests that items can be found by ID
#[tokio::test]
async fn test_get_item_by_user_by_id() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let user_2 = mock_user(&db, "test2@test.com", "test2").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let _item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    // Ensure items for other users aren't included
    let _item_3 = mock_create_item(&db, &user_2, definition_2_name, 5).await;

    let found_item_1 =
        InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_1.item_id)
            .await
            .unwrap();

    assert_eq!(found_item_1, Some(item_1))
}

/// Tests that items from another user cannot be retrieved
#[tokio::test]
async fn test_get_item_by_user_by_id_no_other_user() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let user_2 = mock_user(&db, "test2@test.com", "test2").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let _item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let _item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    // Ensure items for other users aren't included
    let item_3 = mock_create_item(&db, &user_2, definition_2_name, 5).await;

    let found_item_1 =
        InventoryItemsRepository::get_by_user_by_item_id(&db, user.id, item_3.item_id)
            .await
            .unwrap();

    assert!(found_item_1.is_none())
}

/// Tests that items can be found by definition names
#[tokio::test]
async fn test_get_item_by_user_by_definitions() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let user_2 = mock_user(&db, "test2@test.com", "test2").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    // Ensure items for other users aren't included
    let _item_3 = mock_create_item(&db, &user_2, definition_2_name, 5).await;

    let items = InventoryItemsRepository::get_by_user_by_definitions(
        &db,
        user.id,
        &[definition_1_name, definition_2_name],
    )
    .await
    .unwrap();

    assert_eq!(items.len(), 2);

    let item_1_found = items
        .iter()
        .find(|item| item.id == item_1.id)
        .expect("item 1 should exist");
    let item_2_found = items
        .iter()
        .find(|item| item.id == item_2.id)
        .expect("item 1 should exist");

    assert_eq!(&item_1, item_1_found);
    assert_eq!(&item_2, item_2_found);
}

/// Tests that items can be found by definition names only selecting part of the set
#[tokio::test]
async fn test_get_item_by_user_by_definitions_partial() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let user_2 = mock_user(&db, "test2@test.com", "test2").await;

    let definition_1_name = uuid::uuid!("2de1b790-5f80-4cdb-8d61-0d5cef4d5aef");
    let definition_2_name = uuid::uuid!("020924a3-4ede-40f7-a880-2a23135c0fae");

    let item_1 = mock_create_item(&db, &user, definition_1_name, 5).await;
    let _item_2 = mock_create_item(&db, &user, definition_2_name, 5).await;

    // Ensure items for other users aren't included
    let _item_3 = mock_create_item(&db, &user_2, definition_2_name, 5).await;

    let items =
        InventoryItemsRepository::get_by_user_by_definitions(&db, user.id, &[definition_1_name])
            .await
            .unwrap();

    assert_eq!(items.len(), 1);

    let item_1_found = items
        .iter()
        .find(|item| item.id == item_1.id)
        .expect("item 1 should exist");

    assert_eq!(&item_1, item_1_found);
}
