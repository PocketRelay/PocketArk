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
use serde_with::__private__::duplicate_key_impls::PreventDuplicateInsertsMap;
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

#[derive(Default)]
pub struct CurrencyTracker {
    map: HashMap<String, u32>,
}

impl CurrencyTracker {
    pub fn earn_currency(&mut self, name: &str, value: u32) {
        if let Some(existing) = self.map.get_mut(name) {
            *existing += value
        } else {
            self.map.insert(name.to_string(), value);
        }
    }

    pub fn get_currency_adative(&mut self, name: &str, multiplier: f32) -> u32 {
        let value = self.map.get(name).copied().unwrap_or_default();
        (value as f32 * multiplier).trunc() as u32
    }

    pub fn into_inner(self) -> HashMap<String, u32> {
        self.map
    }
}

pub struct RewardBuilder {
    pub name: String,
    pub exp: u32,
    pub currencies: CurrencyTracker,
}

impl RewardBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            exp: 0,
            currencies: CurrencyTracker::default(),
        }
    }

    pub fn add_exp(&mut self, value: u32) -> &mut Self {
        self.exp.saturating_add(value);
        self
    }
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

    let mut xp_earned = 0;

    let mut score = 0;
    let mut total_score = 0;

    let mut badges = Vec::new();
    let mut reward_sources = Vec::new();

    let mut total_currency = CurrencyTracker::default();

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

                total_currency.earn_currency(&badge.currency, currency_reward);
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

    for value in &mission_data.modifiers {
        let modifier = services
            .match_data
            .modifiers
            .iter()
            .find(|v| v.name.eq(&value.name));
        let modifier = match modifier {
            Some(value) => value,
            None => continue,
        };

        let modifier_value = match modifier.values.iter().find(|v| v.name.eq(&value.value)) {
            Some(value) => value,
            None => continue,
        };

        let last_xp = xp_earned;

        if let Some(xp_data) = &modifier_value.xp_data {
            if xp_data.flat_amount > 0 {
                xp_earned += xp_data.flat_amount;
            }

            if xp_data.additive_multiplier > 0.0 {
                let adative = (xp_earned as f32 * xp_data.additive_multiplier).trunc() as u32;
                xp_earned += adative;
            }
        }

        let mut local_earning = CurrencyTracker::default();

        let cur_data = &modifier_value.currency_data;
        for (curr_key, curr_value) in cur_data {
            if curr_value.flat_amount > 0 {
                total_currency.earn_currency(curr_key, curr_value.flat_amount);
                local_earning.earn_currency(curr_key, curr_value.flat_amount);
            }

            if curr_value.additive_multiplier > 0.0 {
                let adative_value =
                    total_currency.get_currency_adative(curr_key, curr_value.additive_multiplier);
                total_currency.earn_currency(curr_key, adative_value);
                local_earning.earn_currency(curr_key, adative_value);
            }
        }

        reward_sources.push(RewardSource {
            name: modifier.name.clone(),
            xp: xp_earned - last_xp,
            currencies: local_earning.into_inner(),
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
