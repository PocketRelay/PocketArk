use pocket_ark_database::{
    DbPool,
    dto::users::{CreateUserDto, NormalizedEmail, UserDto},
    repositories::users::UserRepository,
};

pub use pocket_ark_database::test_memory_database as test_database;

pub async fn mock_user(
    db: &DbPool,
    email: impl AsRef<str>,
    username: impl Into<String>,
) -> UserDto {
    UserRepository::create(
        db,
        CreateUserDto {
            email: NormalizedEmail::new(email),
            username: username.into(),
            password: "test".to_string(),
        },
    )
    .await
    .unwrap()
}
