use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

use crate::http::models::{
    store::{Currency, UserCurrenciesResponse},
    RawJson,
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

/// POST /store/article
pub async fn obtain_article() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Store_Article_Definitions-1688700283519.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// POST /store/unclaimed/claimAll
pub async fn claim_unclaimed() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Store_claim_unclaimed-1688700288596.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
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
