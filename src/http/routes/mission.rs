use crate::{
    http::models::{
        auth::Sku,
        mission::{
            CompleteMissionData, MissionDetails, MissionPlayerData, MissionPlayerInfo,
            PlayerInfoResult, PrestigeProgression, StartMissionRequest, StartMissionResponse,
        },
        HttpError, RawJson,
    },
    services::game::{
        manager::GetGameMessage, GetMissionDataMessage, SetCompleteMissionMessage,
        SetModifiersMessage,
    },
    state::App,
};
use axum::{extract::Path, Json};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

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

    let now = Utc::now();

    let waves = mission_data
        .player_data
        .iter()
        .map(|value| value.waves_completed)
        .max()
        .unwrap_or_default();

    let level: String = mission_data
        .modifiers
        .iter()
        .find(|value| value.name == "level")
        .map(|value| value.value.clone())
        .unwrap_or_else(|| "MPAqua".to_string());
    let difficulty: String = mission_data
        .modifiers
        .iter()
        .find(|value| value.name == "difficulty")
        .map(|value| value.value.clone())
        .unwrap_or_else(|| "bronze".to_string());
    let enemy_type: String = mission_data
        .modifiers
        .iter()
        .find(|value| value.name == "enemytype")
        .map(|value| value.value.clone())
        .unwrap_or_else(|| "outlaw".to_string());

    let players = mission_data
        .player_data
        .into_iter()
        .filter_map(|value| process_player_data(value).ok())
        .collect();

    Ok(Json(MissionDetails {
        sku: Sku::default(),
        name: mission_id.to_string(),
        duration_sec: mission_data.duration_sec,
        percent_complete: mission_data.percent_complete,
        waves_encountered: waves,
        extraction_state: mission_data.extraction_state,
        enemy_type,
        difficulty,
        map: level,
        start: now,
        end: now,
        processed: now,
        player_infos: players,
        modifiers: mission_data.modifiers,
    }))
}

pub fn process_player_data(data: MissionPlayerData) -> Result<MissionPlayerInfo, HttpError> {
    let badges = Vec::new();
    let items_earned = Vec::new();
    let challenges_updated = HashMap::new();
    let reward_sources = Vec::new();
    let prestige_progression = PrestigeProgression {
        before: HashMap::new(),
        after: HashMap::new(),
    };

    let total_currencies_earned = Vec::new();

    let result = PlayerInfoResult {
        challenges_updated,
        items_earned,
        xp_earned: 0,
        previous_xp: 0,
        current_xp: 0,
        previous_level: 0,
        level: 0,
        leveled_up: false,
        score: 0,
        total_score: 0,
        character_class_name: Uuid::new_v4(),
        total_currencies_earned,
        reward_sources,
        prestige_progression,
    };

    let a = MissionPlayerInfo {
        activities_processed: true,
        bonuses: vec![],
        activities: vec![],
        badges,
        stats: data.stats,
        result,
        pid: data.nucleus_id,
        persona_id: data.persona_id,
        persona_display_name: "".to_string(),
        character_id: Uuid::new_v4(),
        character_class: Uuid::new_v4(),
        modifiers: vec![],
        session_id: Uuid::new_v4(),
        wave_participation: data.waves_in_match,
        present_at_end: data.present_at_end,
    };

    Ok(a)
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

#[test]
fn test() {}
