use std::collections::HashMap;

use axum::Json;
use chrono::Utc;

use hyper::StatusCode;
use log::debug;
use serde_json::Map;

use crate::{
    database::entity::{InventoryItem, ValueMap},
    http::models::{
        inventory::{ActivityResult, ItemDefinition},
        store::{
            ClaimUncalimedResponse, Currency, ObtainStoreItemRequest, ObtainStoreItemResponse,
            UpdateSeenArticles, UserCurrenciesResponse,
        },
        RawJson,
    },
    state::App,
};

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
    Json(req): Json<ObtainStoreItemRequest>,
) -> Json<ObtainStoreItemResponse> {
    debug!("Requested buy store article: {:?}", req);

    // TODO:
    // - Check balance can afford
    // - Take balance and give items

    let services = App::services();
    let balance = u32::MAX / 2;
    let currencies = vec![
        Currency {
            name: "MTXCurrency".to_string(),
            balance,
        },
        Currency {
            name: "GrindCurrency".to_string(),
            balance,
        },
        Currency {
            name: "MissionCurrency".to_string(),
            balance,
        },
    ];

    let now = Utc::now();

    let items: Vec<InventoryItem> = vec![InventoryItem {
        id: 0,
        user_id: 1,
        item_id: uuid::uuid!("ac948017-beb4-459d-8861-fab0b950d5da"),
        definition_name: "c5b3d9e6-7932-4579-ba8a-fd469ed43fda".to_string(),
        stack_size: 1,
        seen: false,
        instance_attributes: ValueMap(HashMap::new()),
        created: now,
        last_grant: now,
        earned_by: "granted".to_string(),
        restricted: false,
    }];
    let definitions: Vec<&'static ItemDefinition> = items
        .iter()
        .filter_map(|item| services.defs.inventory.map.get(&item.definition_name))
        .collect();

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
        items_earned: items.clone(),
        item_definitions: definitions.clone(),
        entitlements_granted: vec![],
        prestige_progression_map: Map::new(),
        character_class_name: None,
    };

    Json(ObtainStoreItemResponse {
        generated_activity_result: activity,
        items,
        definitions,
    })
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
pub async fn get_currencies() -> Json<UserCurrenciesResponse> {
    let balance = u32::MAX / 2;
    let list = vec![
        Currency {
            name: "MTXCurrency".to_string(),
            balance,
        },
        Currency {
            name: "GrindCurrency".to_string(),
            balance,
        },
        Currency {
            name: "MissionCurrency".to_string(),
            balance,
        },
    ];

    Json(UserCurrenciesResponse { list })
}
