use crate::{
    http::models::{
        mission::{CompleteMissionData, MissionDetails, StartMissionRequest, StartMissionResponse},
        HttpError, RawJson,
    },
    services::game::{
        manager::GetGameMessage, GetMissionDataMessage, SetCompleteMissionMessage,
        SetModifiersMessage,
    },
    state::App,
};
use axum::{extract::Path, Json};
use hyper::StatusCode;
use log::debug;
use serde_json::Value;

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
pub async fn get_mission(Path(mission_id): Path<u32>) -> Result<Json<MissionDetails>, HttpError> {
    debug!("Requested mission details: {}", mission_id);

    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage {
            game_id: mission_id,
        })
        .await
        .map_err(|_| HttpError::new("Game service down", StatusCode::SERVICE_UNAVAILABLE))?
        .ok_or(HttpError::new("Unknown game", StatusCode::NOT_FOUND))?;

    let mission_data = game
        .send(GetMissionDataMessage)
        .await
        .map_err(|_| HttpError::new("Failed to send message", StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(HttpError::new(
            "Missing mission data",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    Ok(Json(mission_data))
}

/// POST /user/mission/:id/start
///
/// Starts a mission
pub async fn start_mission(
    Path(mission_id): Path<u32>,
    Json(req): Json<StartMissionRequest>,
) -> Result<Json<StartMissionResponse>, HttpError> {
    debug!("Mission started: {} {:?}", mission_id, req);

    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage {
            game_id: mission_id,
        })
        .await
        .map_err(|_| HttpError::new("Game service down", StatusCode::SERVICE_UNAVAILABLE))?
        .ok_or(HttpError::new("Unknown game", StatusCode::NOT_FOUND))?;

    game.send(SetModifiersMessage {
        modifiers: req.modifiers,
    })
    .await
    .map_err(|_| HttpError::new("Failed to set modifiers", StatusCode::INTERNAL_SERVER_ERROR))?;

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
    Json(req): Json<CompleteMissionData>,
) -> Result<StatusCode, HttpError> {
    debug!("Mission finished: {} {:#?}", mission_id, req);

    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage {
            game_id: mission_id,
        })
        .await
        .map_err(|_| HttpError::new("Game service down", StatusCode::SERVICE_UNAVAILABLE))?
        .ok_or(HttpError::new("Unknown game", StatusCode::NOT_FOUND))?;
    game.send(SetCompleteMissionMessage { mission_data: req })
        .await
        .map_err(|_| {
            HttpError::new(
                "Failed to set finished data",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /mission/seen
pub async fn update_seen(Json(req): Json<Value>) -> StatusCode {
    debug!("Update mission seen: {:?}", req);
    StatusCode::NO_CONTENT
}
