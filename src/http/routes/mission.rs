use crate::{
    http::{
        middleware::JsonDump,
        models::{
            errors::{DynHttpError, HttpResult},
            mission::*,
            RawJson,
        },
    },
    services::game::manager::GameManager,
};
use axum::{extract::Path, Extension, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::sync::Arc;

static CURRENT_MISSIONS_DEFINITION: &str =
    include_str!("../../resources/data/currentMissions.json");

/// GET /mission/current
///
/// Obtains a list of currently avaiable missions
pub async fn current_missions() -> RawJson {
    RawJson(CURRENT_MISSIONS_DEFINITION)
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
///
/// TODO: The JSON body given here seems to be not text, likey its been gzipped and
/// isn't being handled on the client correctly
pub async fn finish_mission(
    Path(mission_id): Path<u32>,
    Extension(game_manager): Extension<Arc<GameManager>>,
    JsonDump(req): JsonDump<CompleteMissionData>,
) -> Result<StatusCode, DynHttpError> {
    // TODO: Handling, JSON structure here is possibly incorrect? Got 400 error

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
