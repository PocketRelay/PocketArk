use pocket_ark::database::{
    dto::users::{CreateUserDto, NormalizedEmail, UserDto},
    migrations::{apply_migrations, initialize_migrations_table},
    repositories::users::UserRepository,
};
use sqlx::SqlitePool;

pub async fn test_database() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    initialize_migrations_table(&pool).await.unwrap();

    {
        let mut transaction = pool.begin().await.unwrap();
        apply_migrations(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
    }

    pool
}

pub async fn mock_user(
    db: &SqlitePool,
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
