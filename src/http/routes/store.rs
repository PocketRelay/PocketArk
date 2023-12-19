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
    },
    state::App,
};
use axum::{Extension, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::DatabaseConnection;

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

    // Find the article we are looking for
    let article = services
        .store
        .catalog
        .articles
        .iter()
        .find(|value| value.name.ends_with(&req.article_name))
        .ok_or(HttpError::new("Unknown article", StatusCode::NOT_FOUND))?;

    // Find the item the user is trying to buy from the article
    let article_item = services
        .items
        .items
        .by_name(&article.item_name)
        .ok_or(HttpError::new(
            "Unknown article item",
            StatusCode::NOT_FOUND,
        ))?;

    // Article price for currency
    let price = article
        .prices
        .iter()
        .find(|price| price.currency.eq(&req.currency))
        .ok_or(HttpError::new("Invalid currency", StatusCode::CONFLICT))?;

    // Obtain the user currency
    let user_currencies = Currency::all(&db, &user).await?;

    // Update the currencies (attempting to pay)
    let mut currencies = Vec::with_capacity(user_currencies.len());
    let mut paid: bool = false;
    for mut currency in user_currencies {
        if currency.ty == req.currency && currency.balance >= price.final_price {
            let new_balance = currency.balance - price.final_price;

            currency = currency.update(&db, new_balance).await?;
            paid = true;
        }

        currencies.push(currency);
    }

    if !paid {
        return Err(HttpError::new(
            "Currency balance cannot be less than 0.",
            StatusCode::CONFLICT,
        ));
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
