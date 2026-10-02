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
            shared_data::SharedProgression,
        },
        repositories::{
            challenge_progress::ChallengeProgressRepository, characters::CharactersRepository,
            currency::CurrencyRepository, shared_data::SharedDataRepository, users::UserRepository,
        },
    },
    definitions::{
        activity::ActivityEvent,
        challenges::{ChallengeCounter, ChallengeDefinition, Challenges, CurrencyReward},
        classes::Classes,
        currency::CurrencyType,
        i18n::{I18nDescription, I18nName},
        level_tables::LevelTables,
    },
    http::models::mission::{
        CompleteMissionData, MissionDetails, MissionPlayerData, MissionPlayerInfo, PlayerInfoResult,
    },
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{
            MissionOutcomeProcessor, ProcessMissionData, badges::BadgeProcessor,
            initial_score::InitialScoreProcessor, initial_xp::InitialXpProcessor,
            modifiers::ModifiersProcessor,
        },
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

pub async fn process_player_data(
    db: &mut DbTransaction<'_>,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    debug!("Processing player data");

    let classes = Classes::get();
    let level_tables = LevelTables::get();

    let user = UserRepository::get_by_id(db.deref_mut(), data.nucleus_id as i64)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;

    debug!("Loaded processing user");
    let mut shared_data = SharedDataRepository::get_by_user(db.deref_mut(), user.id).await?;

    debug!("Loaded shared data");

    // Ensure the player actually has a character selected
    let active_character_id = shared_data
        .active_character_id
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let mut character =
        CharactersRepository::get_by_user_by_id(db.deref_mut(), user.id, active_character_id)
            .await?
            .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = classes
        .by_name(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let process_mission_data = ProcessMissionData {
        player_data: data,
        mission_data,
    };
    let mut data_builder = MissionOutcomeDataBuilder::new();

    debug!("Processing score");
    InitialScoreProcessor.process_mission_data(&process_mission_data, &mut data_builder);

    debug!("Processing badges");
    BadgeProcessor.process_mission_data(&process_mission_data, &mut data_builder);

    debug!("Base score reward");
    InitialXpProcessor.process_mission_data(&process_mission_data, &mut data_builder);

    debug!("Compute modifiers");
    ModifiersProcessor.process_mission_data(&process_mission_data, &mut data_builder);

    debug!("Compute leveling");

    // Character leveling
    let level_table = level_tables
        .by_name(&class.level_name)
        .expect("Missing class level table");

    let previous_xp = character.xp;
    let previous_level = character.level;

    let (new_xp, level) =
        level_table.compute_leveling(character.xp, character.level, data_builder.xp_earned);

    let prestige_level_table = level_tables
        .by_name(&class.prestige_level_name)
        .expect("Missing prestige level table");

    debug!("Compute prestige");

    // Insert the initial prestige data if we don't have any
    // (Needs to happen *before* append_prestige_before to ensure it shows up in the "before" state)
    if !shared_data
        .shared_progression
        .iter()
        .any(|value| value.name.eq(&class.prestige_level_name))
    {
        shared_data.shared_progression.push(SharedProgression {
            i18n_name: I18nName::raw(""),
            i18n_description: I18nDescription::raw(""),
            level: 0,
            name: class.prestige_level_name,
            xp: prestige_level_table.initial_progression(),
        });

        SharedDataRepository::set_user_shared_progression(
            db.deref_mut(),
            user.id,
            &shared_data.shared_progression,
        )
        .await?;
    }

    // Insert the before change
    data_builder.append_prestige_before(&shared_data);

    // Character prestige leveling
    {
        let shared_progression = &mut shared_data.shared_progression;
        let prestige_value = shared_progression
            .iter_mut()
            .find(|value| value.name.eq(&class.prestige_level_name));

        // Update the prestige value in-place
        if let Some(prestige_value) = prestige_value {
            let (new_xp, level) = prestige_level_table.compute_leveling(
                prestige_value.xp,
                prestige_value.level,
                data_builder.xp_earned,
            );

            prestige_value.xp = new_xp;
            prestige_value.level = level;

            // Save the changed progression
            SharedDataRepository::set_user_shared_progression(
                db.deref_mut(),
                user.id,
                shared_progression,
            )
            .await?;
        }
    }

    // Insert after change
    data_builder.append_prestige_after(&shared_data);

    data_builder.add_reward_currency("enemytype", CurrencyType::Grind, 0);
    data_builder.add_reward_currency("level_multiplier", CurrencyType::Grind, 0);
    data_builder.add_reward_currency("difficulty_multiplier", CurrencyType::Grind, 0);
    data_builder.add_reward_currency("enemytype_multiplier", CurrencyType::Grind, 0);

    debug!("Process challenges");

    process_challenges(&data.activity_report.activities, &mut data_builder);

    let mut challenges_updated: BTreeMap<String, ChallengeUpdated> = BTreeMap::new();

    // Save challenge changes
    for (index, change) in data_builder.challenges_updates.iter().enumerate() {
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

    // TOD: Character leveling up needs to add 3 skill points per level

    // Update character level and xp
    if new_xp != previous_xp || level > previous_level {
        CharactersRepository::set_xp_level(db.deref_mut(), character.id, new_xp, level).await?;

        character.xp = new_xp;
        character.level = level;
    }

    debug!("Updating currencies");

    // Add all the new currency amounts
    CurrencyRepository::apply_currency_updates(
        db.deref_mut(),
        user.id,
        data_builder
            .total_currency
            .iter()
            .map(|(key, value)| CurrencyUpdateDto {
                ty: *key,
                balance: *value as i32,
            })
            .collect(),
    )
    .await?;

    let total_currencies_earned = data_builder
        .total_currency
        .into_iter()
        .map(|(name, value)| CurrencyReward { name, value })
        .collect();

    let result = PlayerInfoResult {
        challenges_updated,
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
        session_id: user.id.to_string(),
        wave_participation: data.waves_in_match,
        present_at_end: data.present_at_end,
    })
}

/// Temporary data for storing changes to challenges
pub struct ChallengeProgressChange {
    /// The challenge definition
    pub definition: &'static ChallengeDefinition,
    /// The counter to change
    pub counter: &'static ChallengeCounter,
    /// The progress made to the challenge
    pub progress: u32,
}

/// Processes challenge updates that may have occurred from the
/// collection of `activities`
fn process_challenges(activities: &[ActivityEvent], data_builder: &mut MissionOutcomeDataBuilder) {
    let challenge_definitions = Challenges::get();

    activities
        .iter()
        // Find activities with associated challenges
        .filter_map(|activity| {
            let (definition, counter, descriptor) =
                challenge_definitions.get_by_activity(activity)?;
            // Only include activities with current progress
            let progress = activity.attribute_u32(&descriptor.progress_key).ok()?;

            Some((definition, counter, progress))
        })
        .for_each(|(definition, counter, progress)| {
            // Store the challenge changes
            data_builder.add_challenge_progress(ChallengeProgressChange {
                definition,
                counter,
                progress,
            })
        });
}
