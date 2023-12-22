use crate::{
    http::models::user_match::{MatchBadgesResponse, MatchModifiersResponse},
    services::match_data::MatchDataDefinitions,
};
use axum::Json;

/// GET /user/match/badges
///
/// Obtains a list of badge definitions for badges that can
/// be awarded during a multiplayer match
pub async fn get_badges() -> Json<MatchBadgesResponse> {
    let match_data = MatchDataDefinitions::get();
    let list = &match_data.badges;
    Json(MatchBadgesResponse {
        list,
        total_count: list.len(),
    })
}

/// GET /user/match/modifiers
///
/// Obtains a list of modifier definitions that can be applied
/// to a match
pub async fn get_modifiers() -> Json<MatchModifiersResponse> {
    let match_data = MatchDataDefinitions::get();
    let list = &match_data.modifiers;
    Json(MatchModifiersResponse {
        list,
        total_count: list.len(),
    })
}
