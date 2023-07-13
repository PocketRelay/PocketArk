use axum::{extract::Path, Json};
use hyper::StatusCode;
use log::debug;
use serde_json::Value;
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
pub async fn resolve_mission(Path(id): Path<Uuid>) -> StatusCode {
    debug!("Strike team mission resolve: {}", id);

    // TODO: Randomize outcome

    StatusCode::NOT_FOUND
}
