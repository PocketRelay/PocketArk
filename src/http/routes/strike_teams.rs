use crate::http::models::RawJson;

/// GET /striketeams
pub async fn get() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeams.json");
    RawJson(DEFS)
}

/// GET /striketeams/successRate
pub async fn get_success_rate() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeamSuccessRate.json");
    RawJson(DEFS)
}
