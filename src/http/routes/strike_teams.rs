use axum::extract::Path;
use log::debug;
use uuid::Uuid;

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

/// GET /striketeams/missionConfig
pub async fn get_mission_config() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeamMissionConfig.json");
    RawJson(DEFS)
}

/// GET /striketeams/specializations
pub async fn get_specializations() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeamSpecializations.json");
    RawJson(DEFS)
}

/// GET /striketeams/equipment
pub async fn get_equipment() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeamEquipment.json");
    RawJson(DEFS)
}

/// POST /striketeams/:id/mission/resolve
pub async fn resolve_mission(Path(id): Path<Uuid>) -> RawJson {
    debug!("Strike team mission resolve: {}", id);

    // TODO: Randomize outcome

    static DEFS: &str = include_str!("../../resources/data/placeholderStMissionResolve.json");
    RawJson(DEFS)
}

/// POST /striketeams/:id/mission/:id
///
/// Obtain the details about a specific strike team mission
pub async fn get_mission(Path((id, mission_id)): Path<(Uuid, Uuid)>) -> RawJson {
    debug!("Strike team get mission : {} {}", id, mission_id);

    // TODO: Randomize outcome

    static DEFS: &str = include_str!("../../resources/data/strikeTeamMissionSpecific.json");
    RawJson(DEFS)
}

/// POST /striketeams/:id/retire
///
/// Retires (Removes) a strike team from the players
/// strike teams
pub async fn retire(Path(id): Path<Uuid>) {
    debug!("Strike team retire: {}", id);
}

/// POST /striketeams/purchase?currency=MissionCurrency
pub async fn purchase(req: String) {
    debug!("Strike team purchase request: {}", req);
}
