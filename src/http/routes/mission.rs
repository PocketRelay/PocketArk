use crate::{
    database::{
        self,
        entity::{
            challenge_progress::ProgressUpdateType, ChallengeProgress, Character, Currency,
            SharedData, User,
        },
    },
    http::models::{
        auth::Sku,
        character::{LevelTable, Xp},
        mission::{
            ChallengeStatusChange, ChallengeUpdate, CompleteMissionData, MissionDetails,
            MissionModifier, MissionPlayerData, MissionPlayerInfo, PlayerInfoBadge,
            PlayerInfoResult, PrestigeData, PrestigeProgression, RewardSource, StartMissionRequest,
            StartMissionResponse,
        },
        HttpError, RawJson,
    },
    services::{
        game::{
            manager::GetGameMessage, GetMissionDataMessage, SetCompleteMissionMessage,
            SetModifiersMessage,
        },
        match_data::MatchDataService,
    },
    state::App,
};

use axum::{extract::Path, Json};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::{DatabaseConnection, DbErr};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

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

    for value in &mission_data.player_data {
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

pub struct PlayerDataBuilder {
    pub score: u32,
    pub xp_earned: u32,
    pub reward_sources: Vec<RewardSource>,
    pub total_currency: HashMap<String, u32>,
    pub prestige_progression: PrestigeProgression,
    pub items_earned: Vec<Value>,
    pub challenges_updates: HashMap<String, ChallengeUpdate>,
    pub badges: Vec<PlayerInfoBadge>,
}

impl PlayerDataBuilder {
    pub fn new() -> Self {
        Self {
            score: 0,
            xp_earned: 0,
            reward_sources: Vec::new(),
            total_currency: HashMap::new(),
            prestige_progression: PrestigeProgression::default(),
            items_earned: Vec::new(),
            challenges_updates: HashMap::new(),
            badges: Vec::new(),
        }
    }

    fn append_prestige(map: &mut HashMap<Uuid, PrestigeData>, shared_data: &SharedData) {
        // Insert the before change
        shared_data.shared_progression.0.iter().for_each(|value| {
            map.insert(
                value.name,
                PrestigeData {
                    level: value.level,
                    name: value.name,
                    xp: value.xp.current,
                },
            );
        });
    }

    pub fn append_prestige_before(&mut self, shared_data: &SharedData) {
        Self::append_prestige(&mut self.prestige_progression.before, shared_data)
    }
    pub fn append_prestige_after(&mut self, shared_data: &SharedData) {
        Self::append_prestige(&mut self.prestige_progression.after, shared_data)
    }

    pub fn add_reward_xp(&mut self, name: &str, xp: u32) {
        // Ignore adding nothing
        if xp == 0 {
            return;
        }

        // Append earned xp
        self.xp_earned += xp;

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            existing.xp += xp;
        } else {
            self.reward_sources.push(RewardSource {
                currencies: HashMap::new(),
                xp,
                name: name.to_string(),
            });
        }
    }

    pub fn add_reward_currency(&mut self, name: &str, currency: &str, value: u32) {
        // Ignore adding nothing
        if value == 0 {
            return;
        }

        // Append currencies to total currrency

        if let Some(existing) = self.total_currency.get_mut(currency) {
            *existing += value
        } else {
            self.total_currency.insert(currency.to_string(), value);
        }

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            // Update currency within reward

            if let Some(existing) = existing.currencies.get_mut(currency) {
                *existing = existing.saturating_add(value);
            } else {
                existing.currencies.insert(currency.to_string(), value);
            }
        } else {
            let mut currencies = HashMap::new();
            currencies.insert(currency.to_string(), value);

            self.reward_sources.push(RewardSource {
                currencies,
                xp: 0,
                name: name.to_string(),
            });
        }
    }
}

async fn process_player_data(
    db: &'static DatabaseConnection,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    let services = App::services();
    let user = User::get_user(db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;
    let mut shared_data = SharedData::get_from_user(&user, db).await?;

    let character = Character::find_by_id_user(db, &user, shared_data.active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = services
        .defs
        .classes
        .lookup(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let mut data_builder = PlayerDataBuilder::new();

    // Tally up initial base scores awarded from all activities
    data_builder.score = data
        .activity_report
        .activities
        .iter()
        .map(|value| value.attributes.score)
        .sum();

    // Update changed challenges
    for (index, challenge_update) in data
        .activity_report
        .activities
        .iter()
        .filter_map(|activity| services.activity.process_activity(activity))
        .enumerate()
    {
        let (model, update_counter, status_change) =
            ChallengeProgress::handle_update(db, &user, challenge_update).await?;
        let status_change = match status_change {
            ProgressUpdateType::Changed => ChallengeStatusChange::Changed,
            ProgressUpdateType::Created => ChallengeStatusChange::Notify,
        };

        data_builder.challenges_updates.insert(
            (index + 1).to_string(),
            ChallengeUpdate {
                challenge_id: model.challenge_id,
                counters: vec![update_counter],
                status_change,
            },
        );
    }

    // Gives awards and badges for each activity
    data.activity_report
        .activities
        .iter()
        .filter_map(|activity| services.match_data.get_by_activity(activity))
        .for_each(|(badge, progress, levels)| {
            let badge_name = badge.name.to_string();
            let mut xp_reward: u32 = 0;
            let mut currency_reward = 0;
            let mut level_names = Vec::with_capacity(levels.len());

            let level_name = levels.last().map(|value| value.name.to_string());
            levels.into_iter().for_each(|badge_level| {
                xp_reward += badge_level.xp_reward;
                currency_reward += badge_level.currency_reward;
                level_names.push(badge_level.name.clone());
            });

            data_builder.add_reward_xp(&badge_name, xp_reward);
            data_builder.add_reward_currency(&badge_name, &badge.currency, currency_reward);

            data_builder.badges.push(PlayerInfoBadge {
                count: progress,
                level_name,
                rewarded_levels: level_names,
                name: badge.name,
            })
        });

    // Compute modifier amounts
    compute_modifiers(
        &services.match_data,
        &mission_data.modifiers,
        &mut data_builder,
    );

    let level_tables = &services.defs.level_tables;

    // Character leveling
    let level_table = level_tables
        .lookup(&class.level_name)
        .expect("Missing class level table");

    let previous_xp = character.xp;
    let previous_level = character.level;

    let (new_xp, level) = compute_leveling(
        level_table,
        character.xp,
        character.level,
        data_builder.xp_earned,
    );

    // Insert the before change
    data_builder.append_prestige_before(&shared_data);

    // Character prestige leveling
    {
        let level_table = level_tables
            .lookup(&class.prestige_level_name)
            .expect("Missing prestige level table");

        let prestige_value = shared_data
            .shared_progression
            .0
            .iter_mut()
            .find(|value| value.name.eq(&class.prestige_level_name));

        // Update the prestive value in-place
        if let Some(prestige_value) = prestige_value {
            let (new_xp, level) = compute_leveling(
                level_table,
                prestige_value.xp,
                prestige_value.level,
                data_builder.xp_earned,
            );

            prestige_value.xp = new_xp;
            prestige_value.level = level;

            // Save the changed progression
            shared_data = shared_data.save_progression(db).await?;
        }
    }

    // Insert after change
    data_builder.append_prestige_after(&shared_data);

    let character = if new_xp != previous_xp || level > previous_level {
        character.update_xp(db, new_xp, level).await?
    } else {
        character
    };

    let mut total_currencies_earned = Vec::new();
    for (key, value) in data_builder.total_currency {
        let mut currency = Currency::create_or_update(db, &user, key, value).await?;
        currency.balance = value;
        total_currencies_earned.push(currency);
    }

    let result = PlayerInfoResult {
        challenges_updated: data_builder.challenges_updates,
        items_earned: data_builder.items_earned,
        xp_earned: data_builder.xp_earned,
        previous_xp: previous_xp.current,
        current_xp: new_xp.current,
        previous_level,
        level: character.level,
        leveled_up: character.level != previous_level,
        score: data_builder.score,
        total_score: data_builder.score,
        character_class_name: class.name,
        total_currencies_earned,
        reward_sources: data_builder.reward_sources,
        prestige_progression: data_builder.prestige_progression,
    };

    Ok(MissionPlayerInfo {
        activities_processed: true,
        bonuses: vec![],
        activities: vec![],
        badges: data_builder.badges,
        stats: data.stats.clone(),
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
    })
}

/// Computes the xp and currency rewards from the provided match modifiers
/// appending them to the provided data builder
fn compute_modifiers(
    match_data: &MatchDataService,
    modifiers: &[MissionModifier],
    data_builder: &mut PlayerDataBuilder,
) {
    modifiers
        .iter()
        .filter_map(|modifier| match_data.get_modifier_entry(&modifier.name, &modifier.value))
        .for_each(|(modifier, modifier_entry)| {
            if let Some(xp_data) = &modifier_entry.xp_data {
                let amount = xp_data.get_amount(data_builder.xp_earned);
                data_builder.add_reward_xp(&modifier.name, amount);
            }

            modifier_entry
                .currency_data
                .iter()
                .for_each(|(key, modifier_data)| {
                    let amount = modifier_data.get_amount(
                        data_builder
                            .total_currency
                            .get(key)
                            .copied()
                            .unwrap_or_default(),
                    );
                    data_builder.add_reward_currency(&modifier.name, key, amount);
                });
        });
}

/// Computes the new xp and level values from the xp earned in
/// the provided data builder
pub fn compute_leveling(
    level_table: &LevelTable,
    mut xp: Xp,
    mut level: u32,
    xp_earned: u32,
) -> (Xp, u32) {
    xp.current = xp.current.saturating_add(xp_earned);

    while xp.current > xp.next {
        let next_lvl = level_table.get_entry_xp(level + 1);
        if let Some(next_xp) = next_lvl {
            level += 1;

            // Subtract the old next amount from earnings
            xp.current -= xp.next;

            // Assign new next and last values
            xp.last = xp.next;
            xp.next = next_xp;
        }
    }

    (xp, level)
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
