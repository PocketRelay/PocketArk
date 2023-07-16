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
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
};
use rand::{thread_rng, SeedableRng};
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
    // 12 = store pack / tools / nameplates
    // 11 = material loot boxes? (striketeams)
    // 7 = challenge reward?
    // 5 = equipment
    // 4 = consumable
    // 8 = redeemables?
    // 9 = consumable buffs / cap increase?
    // 3 = boosters
    // 1/2:GunType = base weapon mods?
    // 14:GunType = alt weapon mods?
    // 1:{guntype} = base gun unlock
    // 13:{guntype} = alternative gun version unlock
    // {num}:{guntype} = weapons?

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

    // let ids = [
    //     "00deb555-5cb5-4473-ac9a-22e9d1ac2328",
    //     "088efa63-ebdf-4fe5-a52c-0eefa0c92852",
    // ];

    // let items = ids
    //     .into_iter()
    //     .map(|id| InventoryItem::create_item(&user, id.to_string(), 1));

    // let mut items_out = Vec::with_capacity(items.len());
    // for item in items {
    //     let value = item.insert(db).await?;
    //     items_out.push(value);
    // }

    let items_out: Vec<InventoryItem> = give_jumbo_supply_pack(&user).await?;

    let definitions: Vec<&'static ItemDefinition> = items_out
        .iter()
        .filter_map(|item| services.defs.inventory.lookup(&item.definition_name))
        .collect();

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
        .defs
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
