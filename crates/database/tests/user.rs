use pocket_ark_database::{
    dto::users::{CreateUserDto, NormalizedEmail},
    repositories::users::UserRepository,
};

use crate::helpers::test_database;

pub mod helpers;

/// Tests we can create a user
#[tokio::test]
async fn test_create_user() {
    let db = test_database().await;

    let user = UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.email, "test@test.com");
    assert_eq!(user.username, "test");
    assert_eq!(user.password, "test");
}

/// Tests that we cannot create a user if the username is already taken
#[tokio::test]
async fn test_create_user_block_duplicate_username() {
    let db = test_database().await;

    let _user_1 = UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test1.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let err = UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test2.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap_err();

    assert!(err.into_database_error().unwrap().is_unique_violation())
}

/// Tests that we cannot create a user if the email is already taken
#[tokio::test]
async fn test_create_user_block_duplicate_email() {
    let db = test_database().await;
    let _user_1 = UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test1".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let err = UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test2".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(err.into_database_error().unwrap().is_unique_violation())
}

/// Tests that checking if a username is taken returns false when its not in use
#[tokio::test]
async fn test_is_username_taken_not_taken() {
    let db = test_database().await;

    let username_taken = UserRepository::is_username_taken(&db, "test")
        .await
        .unwrap();
    assert!(!username_taken);
}

/// Tests that checking if a username is taken returns true when its in use
#[tokio::test]
async fn test_is_username_taken_taken() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let username_taken = UserRepository::is_username_taken(&db, "test")
        .await
        .unwrap();
    assert!(username_taken);
}

/// Tests that checking if a email is taken returns false when its not in use
#[tokio::test]
async fn test_is_email_taken_not_taken() {
    let db = test_database().await;

    let email_taken = UserRepository::is_email_taken(&db, &NormalizedEmail::new("test@test.com"))
        .await
        .unwrap();
    assert!(!email_taken);
}

/// Tests that checking if a email is taken returns true when its in use
#[tokio::test]
async fn test_is_email_taken_taken() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let email_taken = UserRepository::is_email_taken(&db, &NormalizedEmail::new("test@test.com"))
        .await
        .unwrap();
    assert!(email_taken);
}

/// Tests that we cannot find a user by ID if the ID does not exist
#[tokio::test]
async fn test_get_user_by_id_not_found() {
    let db = test_database().await;
    let user = UserRepository::get_by_id(&db, 1).await.unwrap();
    assert!(user.is_none());
}

/// Tests that we can find a user by ID if the user exists
#[tokio::test]
async fn test_get_user_by_id_found() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let user = UserRepository::get_by_id(&db, 1).await.unwrap();
    assert!(user.is_some());
}

/// Tests that we find the correct user by ID when multiple users exist
#[tokio::test]
async fn test_get_user_by_id_found_correct_target() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test1@test.com"),
            username: "test1".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test2@test.com"),
            username: "test2".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test3@test.com"),
            username: "test3".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let user = UserRepository::get_by_id(&db, 2)
        .await
        .unwrap()
        .expect("user should exist");
    assert_eq!(user.id, 2);
    assert_eq!(user.username, "test2");
    assert_eq!(user.email, "test2@test.com");
}

/// Tests that we cannot find a user by email when they don't exist
#[tokio::test]
async fn test_get_user_by_email_not_found() {
    let db = test_database().await;
    let user = UserRepository::get_by_email(&db, &NormalizedEmail::new("test@test.com"))
        .await
        .unwrap();
    assert!(user.is_none());
}

/// Tests that we can find a user by email when they exist
#[tokio::test]
async fn test_get_user_by_email_found() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test@test.com"),
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let user = UserRepository::get_by_email(&db, &NormalizedEmail::new("test@test.com"))
        .await
        .unwrap();
    assert!(user.is_some());
}

/// Tests that when finding a user by email we pick the correct user when multiple
/// users exist
#[tokio::test]
async fn test_get_user_by_email_found_correct_target() {
    let db = test_database().await;

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test1@test.com"),
            username: "test1".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test2@test.com"),
            username: "test2".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    UserRepository::create(
        &db,
        CreateUserDto {
            email: NormalizedEmail::new("test3@test.com"),
            username: "test3".to_string(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap();

    let user = UserRepository::get_by_email(&db, &NormalizedEmail::new("test2@test.com"))
        .await
        .unwrap()
        .expect("user should exist");
    assert_eq!(user.id, 2);
    assert_eq!(user.username, "test2");
    assert_eq!(user.email, "test2@test.com");
}
