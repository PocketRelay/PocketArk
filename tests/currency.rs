use crate::helpers::{mock_user, test_database};
use pocket_ark::database::{
    dto::{currency::CurrencyUpdateDto, users::UserId},
    repositories::currency::CurrencyRepository,
};
use pocket_ark::definitions::currency::CurrencyType;
use sqlx::SqlitePool;

pub mod helpers;

async fn update_mock_currency(
    db: &SqlitePool,
    user_id: UserId,
    mtx: i32,
    grind: i32,
    mission: i32,
) {
    let mut updates: Vec<CurrencyUpdateDto> = Vec::new();
    if mtx > 0 {
        updates.push(CurrencyUpdateDto {
            ty: CurrencyType::Mtx,
            balance: mtx,
        })
    }

    if grind > 0 {
        updates.push(CurrencyUpdateDto {
            ty: CurrencyType::Grind,
            balance: grind,
        })
    }

    if mission > 0 {
        updates.push(CurrencyUpdateDto {
            ty: CurrencyType::Mission,
            balance: mission,
        })
    }

    CurrencyRepository::apply_currency_updates(db, user_id, updates)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_user_initial_currency() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert!(currency.is_empty());
}

#[tokio::test]
async fn test_user_initialize_currency() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;
    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert!(currency.is_empty());

    CurrencyRepository::add_initial_currency(&db, user.id)
        .await
        .unwrap();

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 3);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, 0);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Grind)
        .expect("grind should exist");

    assert_eq!(grind.balance, 0);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mission)
        .expect("grind should exist");

    assert_eq!(grind.balance, 0);
}

#[tokio::test]
async fn test_user_create_currency_from_update() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    update_mock_currency(&db, user.id, 100, 150, 0).await;

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 2);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, 100);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Grind)
        .expect("grind should exist");

    assert_eq!(grind.balance, 150);
}

#[tokio::test]
async fn test_user_updates_currency_from_update() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    update_mock_currency(&db, user.id, 100, 150, 0).await;
    update_mock_currency(&db, user.id, 100, 150, 0).await;

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 2);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, 200);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Grind)
        .expect("grind should exist");

    assert_eq!(grind.balance, 300);
}

#[tokio::test]
async fn test_user_updates_should_not_exceed_safe_limit() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    update_mock_currency(&db, user.id, 100, 150, 0).await;
    update_mock_currency(&db, user.id, i32::MAX, i32::MAX, 0).await;

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 2);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, CurrencyRepository::MAX_SAFE_CURRENCY as u32);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Grind)
        .expect("grind should exist");

    assert_eq!(grind.balance, CurrencyRepository::MAX_SAFE_CURRENCY as u32);
}

#[tokio::test]
async fn test_user_updates_should_not_exceed_safe_limit_initial() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    update_mock_currency(&db, user.id, i32::MAX, i32::MAX, 0).await;

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 2);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, CurrencyRepository::MAX_SAFE_CURRENCY as u32);

    let grind = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Grind)
        .expect("grind should exist");

    assert_eq!(grind.balance, CurrencyRepository::MAX_SAFE_CURRENCY as u32);
}

#[tokio::test]
async fn test_set_user_currency() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    CurrencyRepository::set_currency_value(&db, user.id, CurrencyType::Mtx, 150)
        .await
        .unwrap();

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 1);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, 150);
}

#[tokio::test]
async fn test_set_user_currency_existing() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    CurrencyRepository::set_currency_value(&db, user.id, CurrencyType::Mtx, 150)
        .await
        .unwrap();

    CurrencyRepository::set_currency_value(&db, user.id, CurrencyType::Mtx, 250)
        .await
        .unwrap();

    let currency = CurrencyRepository::get_by_user(&db, user.id).await.unwrap();
    assert_eq!(currency.len(), 1);

    let mtx = currency
        .iter()
        .find(|currency| currency.ty == CurrencyType::Mtx)
        .expect("mtx should exist");

    assert_eq!(mtx.balance, 250);
}
