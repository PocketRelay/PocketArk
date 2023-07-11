use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

/// GET /store/catalogs
pub async fn get_catalogs() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_Store_Catalog-1688700275563.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
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
pub async fn get_currencies() -> Response {
    let mut resp = include_str!("../../resources/defs/raw/Get_User_Currencies-1688700294409.json")
        .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
