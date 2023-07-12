use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::debug;
use uuid::Uuid;

use crate::http::models::character::CharacterEquipmentList;

/// GET /characters
pub async fn get_characters() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Characters-1689039048546.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /character/:id
///
/// Gets the defintion and details for the character of the provided ID
pub async fn get_character(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_-1689039081314.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active(Path(character_id): Path<Uuid>) -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment
///
/// Gets the current equipment of the provided character
pub async fn get_character_equip(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Equipment_History-1688700230094.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// PUT /character/:id/equipment'
///
/// Updates the equipment for the provided character using
/// the provided equipment list
pub async fn update_character_equip(
    Path(character_id): Path<Uuid>,
    Json(req): Json<CharacterEquipmentList>,
) -> Response {
    debug!("Update charcter equipment: {} - {:?}", character_id, req);

    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment/history
///
/// Obtains the history of the characters previous
/// equipment
pub async fn get_character_equip_history(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Equipment_History-1688700230094.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Update_Character_Skill_Trees-1688700262159.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /character/classes
pub async fn get_classes() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Classes-1688700203514.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /character/levelTables
pub async fn get_level_tables() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Level_Tables-1688700338695.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}
