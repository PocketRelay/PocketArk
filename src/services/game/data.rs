use std::collections::{BTreeMap, HashMap};

use chrono::Utc;
use log::debug;
use sea_orm::{DatabaseConnection, DbErr};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    database::entity::{
        ChallengeProgress, Character, Currency, InventoryItem, SharedData, User,
        challenge_progress::CounterUpdateType, currency::CurrencyType,
    },
    definitions::{
        badges::{BadgeLevelName, Badges},
        challenges::{ChallengeCounter, ChallengeDefinition, Challenges, CurrencyReward},
        classes::Classes,
        level_tables::LevelTables,
        match_modifiers::MatchModifiers,
    },
    http::models::mission::{
        CompleteMissionData, MissionDetails, MissionModifier, MissionPlayerData, MissionPlayerInfo,
        PlayerInfoBadge, PlayerInfoResult, RewardSource,
    },
    services::activity::{
        ActivityEvent, ChallengeStatusChange, ChallengeUpdateCounter, ChallengeUpdated,
        PrestigeData, PrestigeProgression,
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

pub struct PlayerDataBuilder {
    pub score: u32,
    pub xp_earned: u32,
    pub reward_sources: Vec<RewardSource>,
    pub total_currency: HashMap<CurrencyType, u32>,
    pub prestige_progression: PrestigeProgression,
    pub items_earned: Vec<InventoryItem>,
    pub challenges_updates: Vec<ChallengeProgressChange>,
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
            challenges_updates: Vec::new(),
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

    pub fn add_challenge_progress(&mut self, update: ChallengeProgressChange) {
        let existing = self
            .challenges_updates
            .iter_mut()
            // Check if theres already a matching progress update
            .find(|value| {
                value.definition.name == update.definition.name
                    && value.counter.name == update.counter.name
            });

        if let Some(existing) = existing {
            existing.progress = existing.progress.saturating_add(update.progress);
        } else {
            self.challenges_updates.push(update);
        }
    }

    pub fn add_reward_xp(&mut self, name: &str, xp: u32) {
        // Append earned xp
        self.xp_earned = self.xp_earned.saturating_add(xp);

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            existing.xp = existing.xp.saturating_add(xp);
        } else {
            self.reward_sources.push(RewardSource {
                currencies: HashMap::new(),
                xp,
                name: name.to_string(),
            });
        }
    }

    pub fn add_reward_currency(&mut self, name: &str, currency: CurrencyType, value: u32) {
        // Append currencies to total currrency

        if let Some(existing) = self.total_currency.get_mut(&currency) {
            *existing += value
        } else {
            self.total_currency.insert(currency, value);
        }

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            // Update currency within reward

            if let Some(existing) = existing.currencies.get_mut(&currency) {
                *existing = existing.saturating_add(value);
            } else {
                existing.currencies.insert(currency, value);
            }
        } else {
            let mut currencies = HashMap::new();
            currencies.insert(currency, value);

            self.reward_sources.push(RewardSource {
                currencies,
                xp: 0,
                name: name.to_string(),
            });
        }
    }
}

pub async fn process_mission_data(
    db: &DatabaseConnection,
    mission_data: CompleteMissionData,
) -> MissionDetails {
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
        match process_player_data(db, value, &mission_data).await {
            Ok(info) => {
                player_infos.push(info);
            }
            Err(err) => {
                log::error!("Error while processing player: {}", err);
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
        start: now,
        end: now,
        processed: now,
        player_infos,
        modifiers: mission_data.modifiers,
    }
}

pub async fn process_player_data(
    db: &DatabaseConnection,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    debug!("Processing player data");

    let classes = Classes::get();
    let level_tables = LevelTables::get();

    let user = User::by_id(db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;

    debug!("Loaded processing user");
    let mut shared_data = SharedData::get(db, &user).await?;

    debug!("Loaded shared data");

    // Ensure the player actually has a character selected
    let active_character_id = shared_data
        .active_character_id
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let mut character = Character::find_by_id_user(db, &user, active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = classes
        .by_name(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let mut data_builder = PlayerDataBuilder::new();

    debug!("Processing score");

    // Set the initial score from the activity scores
    data_builder.score = data.activity_report.activity_total_score();

    debug!("Processing badges");

    process_badges(&data.activity_report.activities, &mut data_builder);

    debug!("Base score reward");
    // Base reward xp is the score earned
    data_builder.add_reward_xp("base", data_builder.score);

    // TODO: "other_badge_rewards"

    debug!("Compute modifiers");
    // Compute modifier amounts
    compute_modifiers(&mission_data.modifiers, &mut data_builder);

    debug!("Compute leveling");

    // Character leveling
    let level_table = level_tables
        .by_name(&class.level_name)
        .expect("Missing class level table");

    let previous_xp = character.xp;
    let previous_level = character.level;

    let (new_xp, level) =
        level_table.compute_leveling(character.xp, character.level, data_builder.xp_earned);

    debug!("Compute prestige");

    // Insert the before change
    data_builder.append_prestige_before(&shared_data);

    // Character prestige leveling
    {
        let level_table = level_tables
            .by_name(&class.prestige_level_name)
            .expect("Missing prestige level table");

        let prestige_value = shared_data
            .shared_progression
            .0
            .iter_mut()
            .find(|value| value.name.eq(&class.prestige_level_name));

        // Update the prestige value in-place
        if let Some(prestige_value) = prestige_value {
            let (new_xp, level) = level_table.compute_leveling(
                prestige_value.xp,
                prestige_value.level,
                data_builder.xp_earned,
            );

            prestige_value.xp = new_xp;
            prestige_value.level = level;

            // Save the changed progression
            shared_data = shared_data.save_progression(db).await?;
        } else {
            // TODO: Handle appending new shared progression
        }
    }

    // Insert after change
    data_builder.append_prestige_after(&shared_data);

    debug!("Process challenges");

    process_challenges(&data.activity_report.activities, &mut data_builder);

    let mut challenges_updated: BTreeMap<String, ChallengeUpdated> = BTreeMap::new();

    // Save challenge changes
    for (index, change) in data_builder.challenges_updates.iter().enumerate() {
        let (model, counter, change_type) = ChallengeProgress::update(db, &user, change).await?;

        let status_change = match change_type {
            CounterUpdateType::Changed => ChallengeStatusChange::Changed,
            CounterUpdateType::Created => ChallengeStatusChange::Notify,
        };

        // Store the updated challenge
        challenges_updated.insert(
            (index + 1).to_string(),
            ChallengeUpdated {
                challenge_id: model.challenge_id,
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
        character = character.update_xp(db, new_xp, level).await?
    }

    debug!("Updating currencies");

    // Add all the new currency amounts
    Currency::add_many(
        db,
        &user,
        data_builder
            .total_currency
            .iter()
            .map(|(key, value)| (*key, *value)),
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
        character_id: character.id,
        character_class: character.class_name,
        modifiers: vec![],
        session_id: Uuid::new_v4(),
        wave_participation: data.waves_in_match,
        present_at_end: data.present_at_end,
    })
}

/// Processes the `activities` from the game adding any rewards
/// and badges from completed badge levels
fn process_badges(activities: &[ActivityEvent], data_builder: &mut PlayerDataBuilder) {
    let badges = Badges::get();

    activities
        .iter()
        // Find matching badges for the activity
        .filter_map(|activity| {
            // Find a badge matching the activity
            let (badge, progress, levels) = badges.by_activity(activity)?;
            // Only continue if they have a level achieved
            let highest_level = *levels.last()?;

            Some((badge, progress, levels, highest_level))
        })
        .for_each(|(badge, progress, levels, highest_level)| {
            // Total accumulated XP and currency from achieved levels
            let mut total_xp: u32 = 0;
            let mut total_currency: u32 = 0;

            // Names of the levels that have been earned
            let mut level_names: Vec<BadgeLevelName> = Vec::with_capacity(levels.len());

            for level in levels {
                total_xp += level.xp_reward;
                total_currency += level.currency_reward;
                level_names.push(level.name.clone());
            }

            // The reward source is the badge name
            let reward_name = badge.name.to_string();

            // Append the rewards
            data_builder.add_reward_xp(&reward_name, total_xp);
            data_builder.add_reward_currency(&reward_name, badge.currency, total_currency);
            data_builder.badges.push(PlayerInfoBadge {
                count: progress,
                level_name: highest_level.name.clone(),
                rewarded_levels: level_names,
                name: badge.name,
            });
        });
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
fn process_challenges(activities: &[ActivityEvent], data_builder: &mut PlayerDataBuilder) {
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

/// Computes the xp and currency rewards from the provided mission modifiers
/// appending them to the provided data builder
fn compute_modifiers(mission_modifiers: &[MissionModifier], data_builder: &mut PlayerDataBuilder) {
    let match_modifiers = MatchModifiers::get();

    mission_modifiers
        .iter()
        .filter_map(|mission_modifier| {
            // Find a matching modifier
            let match_modifier = match_modifiers.by_name(&mission_modifier.name)?;
            // Find a matching modifier value
            let modifier_value = match_modifier.by_value(&mission_modifier.value)?;

            Some((match_modifier, modifier_value))
        })
        .for_each(|(modifier, modifier_entry)| {
            // Apply xp rewards if the modifier has any
            if let Some(xp_data) = &modifier_entry.xp_data {
                let amount = xp_data.get_amount(data_builder.xp_earned);
                data_builder.add_reward_xp(&modifier.name, amount);
            }

            modifier_entry
                .currency_data
                .iter()
                .for_each(|(key, modifier_data)| {
                    // Get current currency amount for additive multiplier
                    let current_amount = data_builder
                        .total_currency
                        .get(key)
                        .copied()
                        .unwrap_or_default();

                    // Get the earned amount
                    let earned_amount = modifier_data.get_amount(current_amount);
                    data_builder.add_reward_currency(&modifier.name, *key, earned_amount);
                });
        });
}
