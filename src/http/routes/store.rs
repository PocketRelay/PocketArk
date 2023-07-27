use crate::{
    database::entity::{Currency, InventoryItem},
    http::{
        middleware::user::Auth,
        models::{
            store::{
                ClaimUncalimedResponse, ObtainStoreItemRequest, ObtainStoreItemResponse,
                StoreCatalogResponse, UpdateSeenArticles, UserCurrenciesResponse,
            },
            HttpError,
        },
    },
    services::activity::{ActivityItemDetails, ActivityResult},
    state::App,
};
use axum::Json;
use hyper::StatusCode;
use log::debug;

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
    Json(req): Json<ObtainStoreItemRequest>,
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
        .by_name(&article.item_name)
        .ok_or(HttpError::new(
            "Unknown article item",
            StatusCode::NOT_FOUND,
        ))?;

    let db = App::database();

    // Obtain the user currency
    let currencies = Currency::get_from_user(db, &user).await?;
    let _currency = currencies
        .iter()
        .find(|value| value.name == req.currency)
        .ok_or(HttpError::new("Missing currency", StatusCode::BAD_REQUEST))?;

    // TODO: Pre check condition for can afford and allowed within limits

    debug!(
        "Purchased article: {} ({:?})",
        &article_item.name,
        &article_item.locale.name()
    );

    // Create the purchased item
    let mut item = InventoryItem::create_or_append(db, &user, article_item, 1).await?;
    item.stack_size = 1;

    let definitions = vec![article_item];
    let items_earned = vec![item];

    let activity = ActivityResult {
        currencies,
        items: ActivityItemDetails {
            earned: items_earned.clone(),
            definitions: definitions.clone(),
        },
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
pub async fn get_currencies(Auth(user): Auth) -> Result<Json<UserCurrenciesResponse>, HttpError> {
    let db = App::database();
    let currencies = Currency::get_from_user(db, &user).await?;

    Ok(Json(UserCurrenciesResponse { list: currencies }))
}
