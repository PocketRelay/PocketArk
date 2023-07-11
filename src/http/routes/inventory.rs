use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

/// GET /inventory
pub async fn get_inventory() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Inventory_With_Definitions-1688700307239.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /inventory/definitions
pub async fn get_definitions() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Inventory_Definitions-1688700300142.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// PUT /inventory/seen
pub async fn update_inventory_seen() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// POST /inventory/consume
pub async fn consume_inventory() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/PostInventoryConsume.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
