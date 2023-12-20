use crate::{
    http::models::user_match::{MatchBadgesResponse, MatchModifiersResponse},
    services::Services,
};
use axum::Json;

/// GET /user/match/badges
///
/// Obtains a list of badge definitions for badges that can
/// be awarded during a multiplayer match
pub async fn get_badges() -> Json<MatchBadgesResponse> {
    let services = Services::get();
    let list = &services.match_data.badges;
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
    let services = Services::get();
    let list = &services.match_data.modifiers;
    Json(MatchModifiersResponse {
        list,
        total_count: list.len(),
    })
}
