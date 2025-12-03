use crate::{
    database::entity::{StrikeTeamMission, strike_team_mission_progress::UserMissionState},
    http::{
        middleware::{JsonDump, user::Auth},
        models::{
            VecWithCount,
            errors::{DynHttpError, HttpResult},
            mission::*,
            strike_teams::StrikeTeamMissionWithState,
        },
    },
    services::game::{data::process_mission_data, store::Games},
};
use axum::{Extension, Json, extract::Path};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::sync::Arc;

/// GET /mission/current
///
/// Obtains a list of currently available missions
pub async fn current_missions(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<VecWithCount<StrikeTeamMissionWithState>> {
    let current_time = Utc::now().timestamp();
    let missions = StrikeTeamMission::visible_missions(&db, &user, current_time).await?;

    let missions: Vec<StrikeTeamMissionWithState> = missions
        .into_iter()
        .map(|(mission, progress)| match progress {
            Some(value) => StrikeTeamMissionWithState {
                mission,
                user_mission_state: value.user_mission_state,
                seen: value.seen,
                completed: value.completed,
            },
            None => StrikeTeamMissionWithState {
                mission,
                user_mission_state: UserMissionState::Available,
                seen: false,
                completed: false,
            },
        })
        .collect();

    debug!("MISSION LIST: {:?}", missions);

    Ok(Json(VecWithCount::new(missions)))
}

/// GET /user/mission/:id
///
/// Obtains the details about a specific mission
///
/// Called at end of game to obtain information about the
/// game and rewards etc
pub async fn get_mission(
    Path(mission_id): Path<u32>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(games): Extension<Arc<Games>>,
) -> HttpResult<MissionDetails> {
    debug!("Requested mission details: {}", mission_id);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    if let Some(mission_data) = game.read().get_processed_data() {
        return Ok(Json(mission_data));
    }

    let mission_data = game
        .read()
        .get_mission_data()
        .ok_or(MissionError::MissingMissionData)?;

    let mission_data = process_mission_data(&db, mission_data).await;
    game.write().set_processed(mission_data.clone());

    Ok(Json(mission_data))
}

/// POST /user/mission/:id/start
///
/// Starts a mission
pub async fn start_mission(
    Path(mission_id): Path<u32>,
    Extension(games): Extension<Arc<Games>>,
    JsonDump(req): JsonDump<StartMissionRequest>,
) -> HttpResult<StartMissionResponse> {
    debug!("Mission started: {} {:?}", mission_id, req);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    {
        game.write().set_modifiers(req.modifiers);
    }

    let res = StartMissionResponse {
        match_id: mission_id.to_string(),
    };
    Ok(Json(res))
}

/// POST /user/mission/:id/finish
///
/// Submits the details of a mission that has been finished
pub async fn finish_mission(
    Path(mission_id): Path<u32>,
    Extension(games): Extension<Arc<Games>>,
    JsonDump(req): JsonDump<CompleteMissionData>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Mission finished: {} {:#?}", mission_id, req);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    {
        game.write().set_complete_mission(req);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /mission/seen
pub async fn update_seen(JsonDump(req): JsonDump<Value>) -> StatusCode {
    debug!("Update mission seen: {:?}", req);
    StatusCode::NO_CONTENT
}
