use crate::{
    http::models::user_match::{MatchBadgesResponse, MatchModifiersResponse},
    services::{badges::Badges, match_modifiers::MatchModifiers},
};
use axum::Json;

/// GET /user/match/badges
///
/// Obtains a list of badge definitions for badges that can
/// be awarded during a multiplayer match
pub async fn get_badges() -> Json<MatchBadgesResponse> {
    let badges = Badges::get();
    let list = &badges.values;
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
    let modifiers = MatchModifiers::get();
    let list = &modifiers.values;
    Json(MatchModifiersResponse {
        list,
        total_count: list.len(),
    })
}
