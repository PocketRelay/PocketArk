use crate::{
    database::entity::{Character, Currency, InventoryItem},
    http::{
        middleware::user::Auth,
        models::{
            inventory::{ActivityResult, ItemDefinition},
            store::{
                ClaimUncalimedResponse, ObtainStoreItemRequest, ObtainStoreItemResponse,
                UpdateSeenArticles, UserCurrenciesResponse,
            },
            HttpError, RawJson,
        },
    },
    state::App,
};
use axum::Json;
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::ActiveModelTrait;
use serde_json::Map;
use uuid::Uuid;

/// Definition file for the contents of the in-game store
static STORE_CATALOG_DEFINITION: &str = include_str!("../../resources/data/storeCatalog.json");

/// GET /store/catalogs
///
/// Obtains the definitions for the store catalogs. Responds with
/// the store catalog definitions along with all the articles within
/// each catalog
pub async fn get_catalogs() -> RawJson {
    RawJson(STORE_CATALOG_DEFINITION)
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

    let now = Utc::now();

    // categories:
    // 0 = characters
    // 12 = store pack

    let ids = [
        "00deb555-5cb5-4473-ac9a-22e9d1ac2328",
        "088efa63-ebdf-4fe5-a52c-0eefa0c92852",
    ];

    let items = ids
        .into_iter()
        .map(|id| InventoryItem::create_item(&user, id.to_string(), 1));

    let mut items_out = Vec::with_capacity(items.len());
    for item in items {
        let value = item.insert(db).await?;
        items_out.push(value);
    }

    let definitions: Vec<&'static ItemDefinition> = items_out
        .iter()
        .filter_map(|item| services.defs.inventory.lookup(&item.definition_name))
        .collect();

    for item in &items_out {
        let def = definitions
            .iter()
            .find(|value| item.definition_name == value.name);
        if let Some(def) = def {
            if def.category.eq("0") {
                let uuid = Uuid::parse_str(&def.name);
                if let Ok(uuid) = uuid {
                    Character::create_from_item(&services.defs, &user, uuid, db).await?;
                }
            }
        }
    }

    // let items: Vec<InventoryItem> = vec![InventoryItem {
    //     id: 0,
    //     user_id: 1,
    //     item_id: uuid::uuid!("ac948017-beb4-459d-8861-fab0b950d5da"),
    //     definition_name: "c5b3d9e6-7932-4579-ba8a-fd469ed43fda".to_string(),
    //     stack_size: 1,
    //     seen: false,
    //     instance_attributes: ValueMap(Map::new()),
    //     created: now,
    //     last_grant: now,
    //     earned_by: "granted".to_string(),
    //     restricted: false,
    // }];

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
        items_earned: items_out.clone(),
        item_definitions: definitions.clone(),
        entitlements_granted: vec![],
        prestige_progression_map: Map::new(),
        character_class_name: None,
    };

    Ok(Json(ObtainStoreItemResponse {
        generated_activity_result: activity,
        items: items_out,
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
