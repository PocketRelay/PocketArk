use crate::{
    database::entity::{Character, SharedData, User},
    http::models::{
        auth::Sku,
        character::Xp,
        mission::{
            CompleteMissionData, MissionDetails, MissionModifier, MissionPlayerData,
            MissionPlayerInfo, PlayerInfoBadge, PlayerInfoResult, PrestigeProgression,
            RewardSource, StartMissionRequest, StartMissionResponse,
        },
        HttpError, RawJson,
    },
    services::{
        game::{
            manager::GetGameMessage, GetMissionDataMessage, SetCompleteMissionMessage,
            SetModifiersMessage,
        },
        match_data::{Badge, BadgeLevel, MatchModifier, ModifierData},
    },
    state::App,
};
use argon2::password_hash::rand_core::le;
use axum::{extract::Path, Json};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::{DatabaseConnection, DbErr};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tokio::task::{JoinSet, LocalSet};
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

    let db = App::database();
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

    let mut player_infos = Vec::with_capacity(mission_data.player_data.len());

    for value in mission_data.player_data {
        let info = process_player_data(db, value, &mission_data).await;
        if let Ok(info) = info {
            player_infos.push(info);
        }
    }

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
        player_infos,
        modifiers: mission_data.modifiers,
    }))
}

#[derive(Debug, Error)]
enum PlayerDataProcessError {
    #[error("Unknown user")]
    UnknownUser,
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error("Missing character")]
    MissingCharacter,
    #[error("Missing class")]
    MissingClass,
}

async fn process_player_data(
    db: &'static DatabaseConnection,
    data: MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    let services = App::services();
    let user = User::get_user(db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;
    let shared_data = SharedData::get_from_user(&user, db).await?;

    let mut character = Character::find_by_id_user(db, &user, shared_data.active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = services
        .defs
        .classes
        .lookup(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    // Collect modifiers that apply to this match
    let modifier_data: Vec<&MatchModifier> = services
        .match_data
        .modifiers
        .iter()
        .filter(|modif| {
            mission_data
                .modifiers
                .iter()
                .any(|v| v.name.eq(&modif.name))
        })
        .collect();

    let mut xp_earned = 0;

    let mut score = 0;
    let mut total_score = 0;

    let mut badges = Vec::new();
    let mut reward_sources = Vec::new();
    let mut currencies_earned = HashMap::new();

    for activity in data.activity_report.activities {
        score += activity.attributes.score;

        let badge = match services.match_data.get_by_activity(&activity.name) {
            Some(value) => value,
            None => continue,
        };
        let from_act = badge.activities.iter().find(|value| {
            value.activity_name.eq(&activity.name)
                && value.matches_filter(&activity.attributes.extra)
        });

        let from_act = match from_act {
            Some(value) => value,
            None => continue,
        };

        let progress: u32 = match from_act.increment_progress_by.as_str() {
            "count" => activity.attributes.count,
            "score" => activity.attributes.score,
            _ => continue,
        };

        let levels: Vec<&BadgeLevel> = badge
            .levels
            .iter()
            .filter(|value| value.target_count <= progress)
            .collect();
        let last = match levels.last() {
            Some(value) => value,
            None => continue,
        };

        let xp_reward: u32 = levels.iter().map(|value| value.xp_reward).sum();
        let currency_reward: u32 = levels.iter().map(|value| value.currency_reward).sum();

        if xp_reward > 0 || currency_reward > 0 {
            let mut currencies = HashMap::new();
            if currency_reward > 0 {
                currencies.insert(badge.currency.clone(), currency_reward);

                if let Some(value) = currencies_earned.get_mut(&badge.currency) {
                    *value += currency_reward;
                } else {
                    currencies_earned.insert(badge.currency.clone(), currency_reward);
                }
            }

            xp_earned += xp_reward;

            reward_sources.push(RewardSource {
                name: badge.name.to_string(),
                xp: xp_reward,
                currencies,
            })
        }

        let level_names = levels.iter().map(|value| value.name.clone()).collect();

        badges.push(PlayerInfoBadge {
            count: progress,
            level_name: last.name.clone(),
            rewarded_levels: level_names,
            name: badge.name,
        })
    }

    let level_table = services
        .defs
        .level_tables
        .lookup(&class.level_name)
        .expect("Missing class level table");

    let previous_xp = character.xp.current;
    let mut current_xp = previous_xp + xp_earned;

    let previous_level = character.level;
    let mut level = character.level;

    // TODO: account for XP multipliers
    while current_xp > character.xp.next {
        let next_lvl = level_table.get_entry_xp(character.level + 1);
        if let Some(next_xp) = next_lvl {
            level += 1;
            current_xp -= character.xp.next;

            character.xp = Xp {
                current: current_xp,
                last: character.xp.next,
                next: next_xp,
            };
        }
    }
    // TODO: Rewards from modifiers and baselines

    let leveled_up = level > previous_level;

    let items_earned = Vec::new();
    let challenges_updated = HashMap::new();
    let prestige_progression = PrestigeProgression {
        before: HashMap::new(),
        after: HashMap::new(),
    };

    let mut total_currencies_earned = Vec::new();
    // TODO: Update currencies in database and output earnings

    let result = PlayerInfoResult {
        challenges_updated,
        items_earned,
        xp_earned,
        previous_xp,
        current_xp,
        previous_level,
        level,
        leveled_up,
        score,
        total_score,
        character_class_name: class.name,
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
        pid: user.id,
        persona_id: user.id,
        persona_display_name: user.username,
        character_id: character.character_id,
        character_class: character.class_name,
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
