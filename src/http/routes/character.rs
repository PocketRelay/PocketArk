use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};

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
pub async fn get_character() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_-1689039081314.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment
pub async fn get_character_equip() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Equipment_History-1688700230094.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// pUT /character/:id/equipment
pub async fn update_character_equip() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment/history
pub async fn get_character_equip_history() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Equipment_History-1688700230094.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree() -> Response {
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
