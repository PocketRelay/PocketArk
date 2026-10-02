use std::{collections::BTreeMap, ops::DerefMut};

use chrono::Utc;
use log::debug;
use thiserror::Error;

use crate::{
    database::{
        DbErr, DbTransaction,
        dto::{
            challenge_progress::{ChallengeState, CounterUpdateType, CreateChallengeProgressDto},
            currency::CurrencyUpdateDto,
            users::UserDto,
        },
        repositories::{
            challenge_progress::ChallengeProgressRepository, characters::CharactersRepository,
            currency::CurrencyRepository, shared_data::SharedDataRepository, users::UserRepository,
        },
    },
    definitions::{challenges::CurrencyReward, classes::Classes},
    http::models::mission::{
        CompleteMissionData, MissionDetails, MissionPlayerData, MissionPlayerInfo, PlayerInfoResult,
    },
    mission::{
        outcome::{MissionOutcomeDataBuilder, MissionOutcomeLevelingChange},
        processors::{MISSION_PROCESSOR_PIPELINE, ProcessMissionData},
    },
    services::{
        activity::{ChallengeStatusChange, ChallengeUpdateCounter, ChallengeUpdated},
        challenges::apply_challenge_progress_change,
    },
    utils::models::Sku,
};

#[derive(Debug, Error)]
pub enum PlayerDataProcessError {
    #[error("Unknown user")]
    UnknownUser,
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error("Missing character")]
    MissingCharacter,
    #[error("Missing class")]
    MissingClass,
}

pub async fn process_mission_data(
    db: &mut DbTransaction<'_>,
    mission_data: CompleteMissionData,
) -> MissionDetails {
    let _now = Utc::now();

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
        match process_player_data(db, value, &mission_data).await {
            Ok(info) => {
                player_infos.push(info);
            }
            Err(err) => {
                log::error!("Error while processing player: {} {:?}", err, value);
            }
        }
    }

    MissionDetails {
        sku: Sku,
        name: mission_data.match_id,
        duration_sec: mission_data.duration_sec,
        percent_complete: mission_data.percent_complete,
        waves_encountered: waves,
        extraction_state: mission_data.extraction_state,
        enemy_type,
        difficulty,
        map: level,
        // start: now,
        // end: now,
        // processed: now,
        start: "2026-07-11T11:36:01.614+0000".to_string(),
        end: "2026-07-11T11:36:57.096+0000".to_string(),
        processed: "2026-07-11T11:36:57.230+0000".to_string(),
        player_infos,
        modifiers: mission_data.modifiers,
    }
}

async fn process_player_data(
    db: &mut DbTransaction<'_>,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    debug!("Processing player data");

    let classes = Classes::get();

    let user = UserRepository::get_by_id(db.deref_mut(), data.nucleus_id as i64)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;
    debug!("Loaded processing user");

    let shared_data = SharedDataRepository::get_by_user(db.deref_mut(), user.id).await?;
    debug!("Loaded shared data");

    // Ensure the player actually has a character selected
    let active_character_id = shared_data
        .active_character_id
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let character =
        CharactersRepository::get_by_user_by_id(db.deref_mut(), user.id, active_character_id)
            .await?
            .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = classes
        .by_name(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let process_mission_data = ProcessMissionData {
        player_data: data,
        mission_data,
        class,
        character: &character,
        active_character_id,
        shared_data: &shared_data,
    };
    let mut data_builder = MissionOutcomeDataBuilder::new();

    let pipeline = &*MISSION_PROCESSOR_PIPELINE;
    for entry in pipeline {
        debug!(
            "executing mission processing pipeline entry: {}",
            entry.name
        );

        entry
            .processor
            .process_mission_data(&process_mission_data, &mut data_builder);
    }

    let persist_outcome =
        persist_mission_processing_outcome(db, &user, &process_mission_data, &data_builder).await?;

    let (previous_xp, previous_level, current_xp, current_level) = match data_builder.leveling {
        Some(MissionOutcomeLevelingChange { xp, level }) => {
            let previous_xp = character.xp;
            let previous_level = character.level;
            (previous_xp, previous_level, xp, level)
        }
        None => {
            let previous_xp = character.xp;
            let previous_level = character.level;
            (previous_xp, previous_level, previous_xp, previous_level)
        }
    };

    let total_currencies_earned = data_builder
        .total_currency
        .into_iter()
        .map(|(name, value)| CurrencyReward { name, value })
        .collect();

    let result = PlayerInfoResult {
        challenges_updated: persist_outcome.challenges_updated,
        items_earned: data_builder.items_earned,
        xp_earned: data_builder.xp_earned,
        previous_xp: previous_xp.current,
        current_xp: current_xp.current,
        previous_level,
        level: current_level,
        leveled_up: current_level != previous_level,
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
        session_id: user.id.to_string(),
        wave_participation: data.waves_in_match,
        present_at_end: data.present_at_end,
    })
}

struct PersistMissionProcessingOutcomeData {
    challenges_updated: BTreeMap<String, ChallengeUpdated>,
}

/// Persists the changes user data from processing a mission to the database
async fn persist_mission_processing_outcome(
    db: &mut DbTransaction<'_>,
    user: &UserDto,
    process_mission_data: &ProcessMissionData<'_>,
    outcome: &MissionOutcomeDataBuilder,
) -> Result<PersistMissionProcessingOutcomeData, PlayerDataProcessError> {
    let character = &process_mission_data.character;
    let mut challenges_updated: BTreeMap<String, ChallengeUpdated> = BTreeMap::new();

    // Save challenge changes
    for (index, change) in outcome.challenges_updates.iter().enumerate() {
        let challenge_id = change.definition.base.name;
        let challenge = match ChallengeProgressRepository::get_by_user_by_id(
            db.deref_mut(),
            user.id,
            challenge_id,
        )
        .await?
        {
            Some(value) => value,
            None => {
                ChallengeProgressRepository::create(
                    db.deref_mut(),
                    CreateChallengeProgressDto {
                        user_id: user.id,
                        challenge_id,
                        counters: vec![],
                        last_changed: Utc::now(),
                        state: ChallengeState::NotStarted,
                    },
                )
                .await?
            }
        };

        let (update, change_type, counter) = apply_challenge_progress_change(&challenge, change);

        ChallengeProgressRepository::update(db.deref_mut(), user.id, challenge_id, update).await?;

        let status_change = match change_type {
            CounterUpdateType::Changed => ChallengeStatusChange::Changed,
            CounterUpdateType::Created => ChallengeStatusChange::Notify,
        };

        // Store the updated challenge
        challenges_updated.insert(
            (index + 1).to_string(),
            ChallengeUpdated {
                challenge_id,
                counters: vec![ChallengeUpdateCounter {
                    name: counter.name,
                    current_count: counter.current_count,
                }],
                status_change,
            },
        );
    }

    debug!("Saving character level and xp");

    // Update character level and xp
    if let Some(MissionOutcomeLevelingChange { xp, level }) = outcome.leveling {
        CharactersRepository::set_xp_level(db.deref_mut(), character.id, xp, level).await?;
    }

    // Save the changed progression
    SharedDataRepository::set_user_shared_progression(
        db.deref_mut(),
        user.id,
        &outcome.shared_progression,
    )
    .await?;

    debug!("Updating currencies");

    let currency_updates: Vec<CurrencyUpdateDto> = outcome
        .total_currency
        .iter()
        .map(|(key, value)| CurrencyUpdateDto {
            ty: *key,
            balance: *value as i32,
        })
        .collect();

    // Add all the new currency amounts
    CurrencyRepository::apply_currency_updates(db.deref_mut(), user.id, currency_updates).await?;

    Ok(PersistMissionProcessingOutcomeData { challenges_updated })
}
