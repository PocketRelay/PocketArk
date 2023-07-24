use log::error;

use crate::{
    database::{
        entity::{Character, Currency, InventoryItem, User},
        DbResult,
    },
    http::{
        middleware::user::Auth,
        models::{
            inventory::{ActivityResult, ItemDefinition},
            store::{
                ClaimUncalimedResponse, ObtainStoreItemRequest, ObtainStoreItemResponse,
                StoreCatalogResponse, UpdateSeenArticles, UserCurrenciesResponse,
            },
            HttpError, RawJson,
        },
        routes::inventory::get_item_definitions,
    },
    services::{
        items::{Category, GrantedItem},
        store::StoreCatalog,
    },
    state::App,
};
use axum::Json;
use chrono::Utc;
use hyper::StatusCode;
use log::{debug, warn};
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
};
use rand::{thread_rng, SeedableRng};
use sea_orm::ActiveModelTrait;
use serde_json::Map;
use uuid::Uuid;

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

    // TODO:
    // - Check balance can afford
    // - Take balance and give items

    let services = App::services();
    let db = App::database();
    let currencies = Currency::get_from_user(&user, db).await?;

    let currency = currencies
        .iter()
        .find(|value| value.name == req.currency)
        .ok_or(HttpError::new("Missing currency", StatusCode::BAD_REQUEST))?;

    let catalog = &services.store.catalog;

    let article = catalog
        .articles
        .iter()
        .find(|value| value.name.ends_with(&req.article_name))
        .ok_or(HttpError::new("Unknown article", StatusCode::NOT_FOUND))?;

    // TODO: Pre check condition for can afford and allowed within limits

    let article_item =
        services
            .items
            .inventory
            .lookup(&article.item_name)
            .ok_or(HttpError::new(
                "Unknown article item",
                StatusCode::NOT_FOUND,
            ))?;

    debug!(
        "Purchased article: {} ({:?})",
        &article_item.name, &article_item.loc_name
    );

    let mut item = InventoryItem::create_or_append(db, &user, article_item, 1).await?;
    item.stack_size = 1;

    let definitions = vec![article_item];
    let items_earned = vec![item];

    let activity = ActivityResult {
        previous_xp: 0,
        xp: 0,
        xp_gained: 0,
        previous_level: 0,
        level: 0,
        level_up: false,
        challenges_updated_count: 0,
        challenges_completed_count: 0,
        challenges_updated: vec![],
        updated_challenge_ids: vec![],
        news_triggered: 0,
        currencies,
        currency_earned: vec![],
        items_earned: items_earned.clone(),
        item_definitions: definitions.clone(),
        entitlements_granted: vec![],
        prestige_progression_map: Map::new(),
        character_class_name: None,
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
    let currencies = Currency::get_from_user(&user, db).await?;

    Ok(Json(UserCurrenciesResponse { list: currencies }))
}
