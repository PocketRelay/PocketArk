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

    let now = Utc::now();

    let weapon = ["1:AssaultRifle", "1:Pistol", "1:Shotgun", "1:SniperRifle"];
    let alt_weapon = [
        "13:AssaultRifle",
        "13:Pistol",
        "13:Shotgun",
        "13:SniperRifle",
    ];

    let mods = ["2:AssaultRifle", "2:Pistol", "2:Shotgun", "2:SniperRifle"];
    let alt_mods = [
        "14:AssaultRifle",
        "14:Pistol",
        "14:Shotgun",
        "14:SniperRifle",
    ];

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

    // TODO: Aquire item

    // TODO: COnsume

    let mut granted: Vec<GrantedItem> = Vec::new();

    // Item pack consumables
    if article_item.category == Category::ITEM_PACK {
        let pack = services.items.packs.get(&article_item.name);
        if let Some(pack) = pack {
            let mut rng = StdRng::from_entropy();
            if let Err(err) =
                pack.grant_items(&mut rng, services.items.inventory.list(), &mut granted)
            {
                error!("Failed to grant pack items: {} {}", &article_item.name, err);
                return Err(HttpError::new(
                    "Failed to grant pack items",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        } else {
            warn!(
                "Don't know how to handle item pack: {} ({:?})",
                &article_item.name, &article_item.loc_name
            );
            return Err(HttpError::new(
                "Pack item not implemented",
                StatusCode::NOT_IMPLEMENTED,
            ));
        }
    }

    let mut items_out: Vec<InventoryItem> = Vec::with_capacity(granted.len());

    for granted in granted {
        let mut item = InventoryItem::create_or_append(
            db,
            &user,
            granted.defintion.name.to_string(),
            granted.stack_size,
        )
        .await?;
        item.stack_size = granted.stack_size;
        items_out.push(item);
    }

    let definitions = get_item_definitions(&items_out);

    // for item in &items_out {
    //     let def = definitions
    //         .iter()
    //         .find(|value| item.definition_name == value.name);
    //     if let Some(def) = def {
    //         if def.category.eq("0") {
    //             let uuid = Uuid::parse_str(&def.name);
    //             if let Ok(uuid) = uuid {
    //                 Character::create_from_item(&services.defs, &user, uuid, db).await?;
    //             }
    //         }
    //     }
    // }

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

pub async fn give_jumbo_supply_pack(user: &User) -> DbResult<Vec<InventoryItem>> {
    let items = [
        "eaefec2a-d892-498b-a175-e5d2048ae39a", // COBRA RPG
        "af39be6b-0542-4997-b524-227aa41ae2eb", // REVIVE PACK
        "2cc0d932-8e9d-48a6-a6e8-a5665b77e835", // AMMO PACK
        "4d790010-1a79-4bd0-a79b-d52cac068a3a", // FIRST AID PACK
    ];

    const CONSUMABLE_COUNT: u32 = 5;
    const BOOSTERS: &str = "3";
    let mut items_out = Vec::new();

    let services = App::services();
    let db = App::database();

    for item in items {
        let mut item =
            InventoryItem::create_or_append(db, user, item.to_string(), CONSUMABLE_COUNT).await?;
        item.stack_size = CONSUMABLE_COUNT;
        items_out.push(item);
    }

    // Give 5 random boosters
    let mut rand = StdRng::from_entropy();
    let boosters: Vec<&'static ItemDefinition> = services
        .items
        .inventory
        .list()
        .iter()
        .filter(|value| value.category.eq(BOOSTERS))
        .choose_multiple(&mut rand, 5);
    for item in boosters {
        let mut item = InventoryItem::create_or_append(db, user, item.name.clone(), 1).await?;
        item.stack_size = 1;
        items_out.push(item);
    }

    Ok(items_out)
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
