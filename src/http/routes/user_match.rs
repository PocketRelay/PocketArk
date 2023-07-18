use axum::Json;

use crate::{
    http::models::{user_match::MatchBadgesResponse, RawJson},
    state::App,
};

/// GET /user/match/badges
///
/// Obtains a list of badge definitions for badges that can
/// be awarded during a multiplayer match
pub async fn get_badges() -> Json<MatchBadgesResponse> {
    let services = App::services();
    let list = &services.match_data.badges;
    Json(MatchBadgesResponse {
        list,
        total_count: list.len(),
    })
}

/// Definition file for the available match modifiers
static MATCH_MODIFIER_DEFINITIONS: &str = include_str!("../../resources/data/matchModifiers.json");

/// GET /user/match/modifiers
///
/// Obtains a list of modifier definitions that can be applied
/// to a match
pub async fn get_modifiers() -> RawJson {
    RawJson(MATCH_MODIFIER_DEFINITIONS)
}
