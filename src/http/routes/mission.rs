use crate::{
    database::entity::{strike_team_mission_progress::UserMissionState, StrikeTeamMission},
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            errors::{DynHttpError, HttpResult},
            mission::*,
            strike_teams::StrikeTeamMissionWithState,
            VecWithCount,
        },
    },
    services::game_manager::GameManager,
};
use axum::{extract::Path, Extension, Json};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::sync::Arc;

/// GET /mission/current
///
/// Obtains a list of currently avaiable missions
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
    Extension(game_manager): Extension<Arc<GameManager>>,
) -> HttpResult<MissionDetails> {
    debug!("Requested mission details: {}", mission_id);

    let game = game_manager
        .get_game(mission_id)
        .await
        .ok_or(MissionError::UnknownGame)?;

    let game = &mut *game.write().await;

    let mission_data = game
        .get_mission_details(&db)
        .await
        .ok_or(MissionError::MissingMissionData)?;

    Ok(Json(mission_data))
}

/// POST /user/mission/:id/start
///
/// Starts a mission
pub async fn start_mission(
    Path(mission_id): Path<u32>,
    Extension(game_manager): Extension<Arc<GameManager>>,
    JsonDump(req): JsonDump<StartMissionRequest>,
) -> HttpResult<StartMissionResponse> {
    debug!("Mission started: {} {:?}", mission_id, req);

    let game = game_manager
        .get_game(mission_id)
        .await
        .ok_or(MissionError::UnknownGame)?;

    {
        let game = &mut *game.write().await;
        game.set_modifiers(req.modifiers);
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
    Extension(game_manager): Extension<Arc<GameManager>>,
    JsonDump(req): JsonDump<CompleteMissionData>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Mission finished: {} {:#?}", mission_id, req);

    let game = game_manager
        .get_game(mission_id)
        .await
        .ok_or(MissionError::UnknownGame)?;

    {
        let game = &mut *game.write().await;
        game.set_complete_mission(req)
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /mission/seen
pub async fn update_seen(JsonDump(req): JsonDump<Value>) -> StatusCode {
    debug!("Update mission seen: {:?}", req);
    StatusCode::NO_CONTENT
}
