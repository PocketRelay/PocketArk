use crate::{
    database::entity::{currency::CurrencyType, Currency, User},
    definitions::store_catalogs::StoreCatalogs,
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            store::{
                ClaimUncalimedResponse, ObtainStoreItemRequest, ObtainStoreItemResponse,
                StoreCatalogResponse, StoreError, UpdateSeenArticles, UserCurrenciesResponse,
            },
            CurrencyError, DynHttpError, HttpResult,
        },
    },
    services::activity::{ActivityEvent, ActivityName, ActivityResult, ActivityService},
};
use axum::{Extension, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};

/// GET /store/catalogs
///
/// Obtains the definitions for the store catalogs. Responds with
/// the store catalog definitions along with all the articles within
/// each catalog
pub async fn get_catalogs() -> Json<StoreCatalogResponse> {
    let catalogs = StoreCatalogs::get();

    Json(StoreCatalogResponse {
        list: vec![&catalogs.catalog],
    })
}

/// PUT /store/article/seen
///
/// Updates the seen status of a specific store article
pub async fn update_seen_articles(Json(req): Json<UpdateSeenArticles>) -> StatusCode {
    debug!("Update seen articles: {:?}", req);

    // This is no-op, this implementation doesn't store article seen states. However
    // this might change at some point.

    StatusCode::NO_CONTENT
}

/// Attempts to spend the provided `amount` of the specified `currency`
/// for the provided `user`
async fn spend_currency<C>(
    db: &C,
    user: &User,
    currency: CurrencyType,
    amount: u32,
) -> Result<(), DynHttpError>
where
    C: ConnectionTrait + Send,
{
    // Ensure the user owns some of the currency
    let currency = Currency::get(db, user, currency)
        .await?
        // User doesn't have any of the requested currency
        .ok_or(CurrencyError::InsufficientCurrency)?;

    // Ensure they can afford the price
    if currency.balance < amount {
        return Err(CurrencyError::InsufficientCurrency.into());
    }

    let new_balance = currency.balance - amount;

    // Take the price from the currency balance
    currency.update(db, new_balance).await?;

    Ok(())
}

/// POST /store/article
///
/// User request to purchase an item from the in-game store
pub async fn obtain_article(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<ObtainStoreItemRequest>,
) -> HttpResult<ObtainStoreItemResponse> {
    let catalogs = StoreCatalogs::get();

    // Find the article we are looking for
    let article = catalogs
        .catalog
        .get_article(&req.article_name)
        .ok_or(StoreError::UnknownArticle)?;

    // Find the price in the specified currency
    let price = article
        .price_by_currency(req.currency)
        .ok_or(CurrencyError::InvalidCurrency)?;

    let result: ActivityResult = db
        .transaction(|db| {
            Box::pin(async move {
                // Spend the cost of the article
                spend_currency(db, &user, req.currency, price.final_price).await?;

                // Create the activity event
                let event = ActivityEvent::new(ActivityName::ArticlePurchased)
                    .with_attribute("currencyName", req.currency.to_string())
                    .with_attribute("articleName", article.name)
                    .with_attribute("count", 1);

                // Process the event
                ActivityService::process_event(db, &user, event)
                    .await
                    .map_err(Into::<DynHttpError>::into)
            })
        })
        .await?;

    Ok(Json(ObtainStoreItemResponse {
        items: result.items_earned.clone(),
        definitions: result.item_definitions.clone(),
        generated_activity_result: result,
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
) -> HttpResult<UserCurrenciesResponse> {
    let currencies = Currency::all(&db, &user).await?;

    Ok(Json(UserCurrenciesResponse { list: currencies }))
}
