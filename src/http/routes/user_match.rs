use crate::http::models::RawJson;

/// Definition file for the available match badges
static MATCH_BADGE_DEFINITIONS: &str = include_str!("../../resources/data/matchBadges.json");

/// GET /user/match/badges
///
/// Obtains a list of badge definitions for badges that can
/// be awarded during a multiplayer match
pub async fn get_badges() -> RawJson {
    RawJson(MATCH_BADGE_DEFINITIONS)
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
