use crate::{
    database::entity::{Character, Currency, InventoryItem},
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            store::{
                ClaimUncalimedResponse, ObtainStoreItemRequest, ObtainStoreItemResponse,
                StoreCatalogResponse, UpdateSeenArticles, UserCurrenciesResponse,
            },
            HttpError,
        },
    },
    services::{
        activity::ActivityResult,
        items::{BaseCategory, Category},
        store::StoreService,
    },
    state::App,
};
use axum::{Extension, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::{DatabaseConnection, DbErr};
use thiserror::Error;

/// GET /store/catalogs
///
/// Obtains the definitions for the store catalogs. Responds with
/// the store catalog definitions along with all the articles within
/// each catalog
pub async fn get_catalogs() -> Json<StoreCatalogResponse> {
    let services = App::services();
    let catalog = &services.store.catalog;

    // TODO: Catalog seen states loaded from db and added to response

    Json(StoreCatalogResponse {
        list: vec![catalog],
    })
}

/// PUT /store/article/seen
///
/// Updates the seen status of a specific store article
pub async fn update_seen_articles(Json(req): Json<UpdateSeenArticles>) -> StatusCode {
    debug!("Update seen articles: {:?}", req);
    StatusCode::NO_CONTENT
}

#[derive(Debug, Error)]
pub enum StoreError {
    /// Couldn't find the article requested
    #[error("Unknown article")]
    UnknownArticle,
    /// Server definition error, article associated item was
    /// not present in the item definitions
    #[error("Unknown article item")]
    UnknownArticleItem,
    /// Article cannot be purchased with the requested currency
    #[error("Invalid currency")]
    InvalidCurrency,
    /// Database error occurred
    #[error("Server error")]
    Database(#[from] DbErr),
    /// User doesn't have enough currency to purchase the item
    #[error("Currency balance cannot be less than 0.")]
    InsufficientCurrency,
}

impl From<StoreError> for HttpError {
    fn from(value: StoreError) -> Self {
        let reason = value.to_string();
        let status = match value {
            StoreError::UnknownArticle => StatusCode::NOT_FOUND,
            StoreError::InvalidCurrency | StoreError::InsufficientCurrency => StatusCode::CONFLICT,
            StoreError::UnknownArticleItem | StoreError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        HttpError::new_owned(reason, status)
    }
}

/// POST /store/article
///
/// Purchases an item from the store returning the results
/// of the purchase
pub async fn obtain_article(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<ObtainStoreItemRequest>,
) -> Result<Json<ObtainStoreItemResponse>, HttpError> {
    debug!("Requested buy store article: {:?}", req);

    let services = App::services();
    let store_service = &services.store;
    let items_service = &services.items;

    // Find the article we are looking for
    let article = store_service
        .catalog
        .get_article(&req.article_name)
        .ok_or(StoreError::UnknownArticle)?;

    // Find the item given by the article
    let article_item = items_service
        .items
        .by_name(&article.item_name)
        .ok_or(StoreError::UnknownArticleItem)?;

    // Find the price in the specified currency
    let price = article
        .price_by_currency(req.currency)
        .ok_or(StoreError::InvalidCurrency)?;

    // Find the currency to pay with
    {
        let currency = Currency::get(&db, &user, req.currency)
            .await?
            // User doesn't have any of the requested currency
            .ok_or(StoreError::InsufficientCurrency)?;

        // Ensure they can afford the price
        if currency.balance < price.final_price {
            return Err(StoreError::InsufficientCurrency.into());
        }

        let new_balance = currency.balance - price.final_price;

        // Take the price from the currency balance
        currency.update(&db, new_balance).await?;
    }

    // TODO: Pre check condition for can afford and allowed within limits

    debug!(
        "Purchased article: {} ({:?})",
        &article_item.name,
        &article_item.locale.name()
    );

    // Create the purchased item
    let mut item =
        InventoryItem::add_item(&db, &user, article_item.name, 1, article_item.capacity).await?;
    item.stack_size = 1;

    // Handle character creation if the item is a character item
    if article_item
        .category
        .is_within(&Category::Base(BaseCategory::Characters))
    {
        let services = App::services();
        Character::create_from_item(&db, &services.character, &user, &article_item.name).await?;
    }

    // Collect all the currency amounts for the activity result
    let currencies = Currency::all(&db, &user).await?;
    let definitions = vec![article_item];
    let items_earned = vec![item];

    let activity = ActivityResult {
        currencies,
        items_earned: items_earned.clone(),
        item_definitions: definitions.clone(),

        ..Default::default()
    };

    Ok(Json(ObtainStoreItemResponse {
        generated_activity_result: activity,
        items: items_earned,
        definitions,
    }))
}

/// POST /store/unclaimed/claimAll
///
/// Possibly claims earned items from end of match?
pub async fn claim_unclaimed() -> Json<ClaimUncalimedResponse> {
    Json(ClaimUncalimedResponse {
        claim_results: vec![],
        results_complete: true,
    })
}

/// GET /user/currencies
///
/// Response with the balance the user has in each type
/// of digital currency within the game
pub async fn get_currencies(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<UserCurrenciesResponse>, HttpError> {
    let currencies = Currency::all(&db, &user).await?;

    Ok(Json(UserCurrenciesResponse { list: currencies }))
}
