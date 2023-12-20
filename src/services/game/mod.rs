use self::manager::GameManager;
use super::{
    activity::{ActivityEvent, PrestigeData, PrestigeProgression},
    challenges::{ChallengeCounter, ChallengeDefinition, ChallengesService, CurrencyReward},
    match_data::MatchDataService,
};
use crate::{
    blaze::{
        components::{self, game_manager, user_sessions::PLAYER_SESSION_TYPE},
        models::{
            game_manager::{
                AttributesChange, GameSetupContext, GameSetupResponse, NotifyGameReplay,
                NotifyGameStateChange, NotifyPostJoinedGame, PlayerAttributesChange, PlayerRemoved,
                RemoveReason,
            },
            PlayerState,
        },
        packet::Packet,
        session::{NetData, SessionNotifyHandle, WeakSessionLink},
    },
    database::entity::{
        challenge_progress::CounterUpdateType, currency::CurrencyType, users::UserId,
        ChallengeProgress, Character, Currency, InventoryItem, SharedData, User,
    },
    http::models::mission::{
        CompleteMissionData, MissionDetails, MissionModifier, MissionPlayerData, MissionPlayerInfo,
        PlayerInfoBadge, PlayerInfoResult, RewardSource,
    },
    services::activity::{ChallengeStatusChange, ChallengeUpdateCounter, ChallengeUpdated},
    state::App,
    utils::models::Sku,
};
use chrono::Utc;
use log::{debug, error};
use sea_orm::{DatabaseConnection, DbErr};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Weak},
};
use tdf::{ObjectId, TdfMap};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod manager;

pub type GameID = u32;
pub type GameRef = Arc<RwLock<Game>>;
pub type WeakGameRef = Weak<RwLock<Game>>;

pub struct Game {
    /// Unique ID for this game
    pub id: GameID,
    /// The current game state
    pub state: u8,
    /// The current game setting
    pub settings: u32,
    /// The game attributes
    pub attributes: AttrMap,
    /// The list of players in this game
    pub players: Vec<Player>,

    pub modifiers: Vec<MissionModifier>,
    pub mission_data: Option<CompleteMissionData>,
    pub processed_data: Option<MissionDetails>,

    /// Services access
    pub game_manager: Arc<GameManager>,
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

async fn process_player_data(
    db: DatabaseConnection,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    debug!("Processing player data");

    let services = App::services();
    let character_service = &services.character;

    let user = User::get_user(&db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;

    debug!("Loaded processing user");
    let mut shared_data = SharedData::get(&db, &user).await?;

    debug!("Loaded shared data");

    // Ensure the player actually has a character selected
    let active_character_id = shared_data
        .active_character_id
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let mut character = Character::find_by_id_user(&db, &user, active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = character_service
        .classes
        .by_name(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let mut data_builder = PlayerDataBuilder::new();

    debug!("Processing score");

    // Set the initial score from the activity scores
    data_builder.score = data.activity_report.activity_total_score();

    debug!("Processing badges");

    process_badges(
        &data.activity_report.activities,
        &services.match_data,
        &mut data_builder,
    );

    debug!("Base score reward");
    // Base reward xp is the score earned
    data_builder.add_reward_xp("base", data_builder.score);

    // TODO: "other_badge_rewards"

    debug!("Compute modifiers");
    // Compute modifier amounts
    compute_modifiers(
        &services.match_data,
        &mission_data.modifiers,
        &mut data_builder,
    );

    debug!("Compute leveling");

    // Character leveling
    let level_table = services
        .character
        .level_table(&class.level_name)
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
        let level_table = services
            .character
            .level_table(&class.prestige_level_name)
            .expect("Missing prestige level table");

        let prestige_value = shared_data
            .shared_progression
            .0
            .iter_mut()
            .find(|value| value.name.eq(&class.prestige_level_name));

        // Update the prestive value in-place
        if let Some(prestige_value) = prestige_value {
            let (new_xp, level) = level_table.compute_leveling(
                prestige_value.xp,
                prestige_value.level,
                data_builder.xp_earned,
            );

            prestige_value.xp = new_xp;
            prestige_value.level = level;

            // Save the changed progression
            shared_data = shared_data.save_progression(&db).await?;
        } else {
            // TODO: Handle appending new shared progression
        }
    }

    // Insert after change
    data_builder.append_prestige_after(&shared_data);

    debug!("Process challenges");

    process_challenges(
        &data.activity_report.activities,
        &services.challenges,
        &mut data_builder,
    );

    let mut challenges_updated: BTreeMap<String, ChallengeUpdated> = BTreeMap::new();

    // Save challenge changes
    for (index, change) in data_builder.challenges_updates.iter().enumerate() {
        let (model, counter, change_type) = ChallengeProgress::update(&db, &user, change).await?;

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

    // Update character level and xp
    if new_xp != previous_xp || level > previous_level {
        character = character.update_xp(&db, new_xp, level).await?
    }

    debug!("Updating currencies");

    // Add all the new currency amounts
    Currency::add_many(
        &db,
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
fn process_badges(
    activities: &[ActivityEvent],
    match_data: &MatchDataService,
    data_builder: &mut PlayerDataBuilder,
) {
    activities
        .iter()
        // Find matching badges for the activity
        .filter_map(|activity| {
            // Find a badge matching the activity
            let (badge, progress, levels) = match_data.get_by_activity(activity)?;
            // Only continue if they have a level achieved
            let highest_level = *levels.last()?;

            Some((badge, progress, levels, highest_level))
        })
        .for_each(|(badge, progress, levels, highest_level)| {
            // Total accumulated XP and currency from achieved levels
            let mut total_xp: u32 = 0;
            let mut total_currency: u32 = 0;

            // Names of the levels that have been earned
            let mut level_names: Vec<String> = Vec::with_capacity(levels.len());

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
                level_name: highest_level.name.to_string(),
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
fn process_challenges(
    activities: &[ActivityEvent],
    challenge_service: &'static ChallengesService,
    data_builder: &mut PlayerDataBuilder,
) {
    activities
        .iter()
        // Find activities with associated challenges
        .filter_map(|activity| {
            let (definition, counter, descriptor) = challenge_service.get_by_activity(activity)?;
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
                    data_builder.add_reward_currency(&modifier.name, *key, amount);
                });
        });
}

pub const DEFAULT_FIT: u16 = 21600;

impl Game {
    pub const MAX_PLAYERS: usize = 4;

    pub fn new(
        id: u32,
        attributes: TdfMap<String, String>,
        game_manager: Arc<GameManager>,
    ) -> Game {
        Self {
            id,
            state: 1,
            settings: 262144,
            attributes,
            players: Vec::with_capacity(4),
            modifiers: Vec::new(),
            mission_data: None,
            processed_data: None,
            game_manager,
        }
    }

    pub fn set_attributes(&mut self, attributes: AttrMap) {
        let packet = Packet::notify(
            game_manager::COMPONENT,
            game_manager::GAME_ATTR_UPDATE,
            AttributesChange {
                id: self.id,
                attributes: &attributes,
            },
        );

        self.attributes.insert_presorted(attributes.into_inner());

        debug!("Updated game attributes");

        self.notify_all(packet);
    }

    pub fn set_player_attributes(&mut self, user_id: UserId, attributes: AttrMap) {
        let packet = Packet::notify(
            game_manager::COMPONENT,
            game_manager::PLAYER_ATTR_UPDATE,
            PlayerAttributesChange {
                game_id: self.id,
                user_id,
                attributes: &attributes,
            },
        );

        debug!("Updated player attributes");

        self.notify_all(packet);

        let player = self
            .players
            .iter_mut()
            .find(|player| player.user.id == user_id);

        if let Some(player) = player {
            player.attr.insert_presorted(attributes.into_inner());
        }
    }

    pub fn set_complete_mission(&mut self, mission_data: CompleteMissionData) {
        self.mission_data = Some(mission_data);
        self.processed_data = None;
    }

    pub fn set_modifiers(&mut self, modifiers: Vec<MissionModifier>) {
        self.modifiers = modifiers;
    }

    pub async fn get_mission_details(&mut self, db: &DatabaseConnection) -> Option<MissionDetails> {
        if let Some(processed) = self.processed_data.clone() {
            return Some(processed);
        }

        let mission_data = self.mission_data.clone()?;

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
            match process_player_data(db.clone(), value, &mission_data).await {
                Ok(info) => {
                    player_infos.push(info);
                }
                Err(err) => {
                    error!("Error while processing player: {}", err);
                }
            }
        }

        let data = MissionDetails {
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
        };

        self.processed_data = Some(data.clone());

        Some(data)
    }

    pub fn set_state(&mut self, state: u8) {
        self.state = state;

        debug!("Updated game state (Value: {:?})", &state);

        self.notify_all(Packet::notify(
            game_manager::COMPONENT,
            game_manager::GAME_STATE_CHANGE,
            NotifyGameStateChange {
                game_id: self.id,
                state,
            },
        ));
    }

    /// Called by the game manager service once this game has been stopped and
    /// removed from the game list
    fn stopped(self) {
        debug!("Game is stopped (GID: {})", self.id);
    }

    fn stop(&mut self) {
        // Mark the game as stopping
        // self.state = GameState::Destructing;

        let game_manager = self.game_manager.clone();
        // Remove the stopping game
        let game_id = self.id;
        tokio::spawn(async move {
            game_manager.remove_game(game_id).await;
        });
    }

    pub fn remove_player(&mut self, user_id: u32, reason: RemoveReason) {
        // Already empty game handling
        if self.players.is_empty() {
            self.stop();
            return;
        }

        // Find the player index
        let index = self.players.iter().position(|v| v.user.id == user_id);

        let index = match index {
            Some(value) => value,
            None => return,
        };

        // Remove the player
        let player = self.players.remove(index);

        // Set current game of this player
        player.try_clear_game();

        // Update the other players
        self.notify_player_removed(&player, reason);
        // self.notify_fetch_data(&player);
        // self.modify_admin_list(player.player.id, AdminListOperation::Remove);

        debug!(
            "Removed player from game (PID: {}, GID: {})",
            player.user.id, self.id
        );

        // If the player was in the host slot attempt migration
        if index == 0 {
            // self.try_migrate_host();
        }

        if self.players.is_empty() {
            // Game is empty stop it
            self.stop();
        }
    }

    pub fn add_player(&mut self, player: Player, context: GameSetupContext) -> usize {
        let slot = self.players.len();

        self.players.push(player);

        // Obtain the player that was just added
        let player = self
            .players
            .last()
            .expect("Player was added but is missing from players");

        // NOTIFY PLAYER JOINING
        // Notify other players of the joined player
        // self.notify_all(
        //     Components::GameManager(GameManager::PlayerJoining),
        //     PlayerJoining {
        //         slot,
        //         player,
        //         game_id: self.id,
        //     },
        // );

        // Update other players with the client details
        self.add_user_sub(player);

        player.notify(Packet::notify(
            game_manager::COMPONENT,
            game_manager::GAME_SETUP,
            GameSetupResponse {
                game: self,
                context,
            },
        ));

        player.notify(Packet::notify(
            4,
            11,
            NotifyPostJoinedGame {
                game_id: self.id,
                player_id: player.user.id,
            },
        ));

        slot
    }

    pub fn notify_game_replay(&self) {
        self.notify_all(Packet::notify(
            4,
            113,
            NotifyGameReplay {
                game_id: self.id,
                grid: self.id,
            },
        ));
    }

    /// Notifies all the session and the removed session that a
    /// session was removed from the game.
    ///
    /// `player`    The player that was removed
    /// `player_id` The player ID of the removed player
    fn notify_player_removed(&self, player: &Player, reason: RemoveReason) {
        let packet = Packet::notify(
            components::game_manager::COMPONENT,
            components::game_manager::PLAYER_REMOVED,
            PlayerRemoved {
                cntx: 0,
                game_id: self.id,
                player_id: player.user.id,
                reason,
            },
        );
        self.notify_all(packet.clone());
        player.notify(packet);

        self.rem_user_sub(player);
    }

    /// Writes the provided packet to all connected sessions.
    /// Does not wait for the write to complete just waits for
    /// it to be placed into each sessions write buffers.
    ///
    /// `packet` The packet to write
    fn notify_all(&self, packet: Packet) {
        self.players
            .iter()
            .for_each(|value| value.notify(packet.clone()));
    }

    /// Creates a subscription between all the users and the the target player
    fn add_user_sub(&self, target: &Player) {
        debug!("Adding user subscriptions");

        // Subscribe all the clients to eachother
        self.players
            .iter()
            .filter(|other| other.user.id != target.user.id)
            .for_each(|other| {
                target.try_subscribe(other.user.id, other.notify_handle.clone());
                other.try_subscribe(target.user.id, target.notify_handle.clone());
            });
    }

    /// Notifies the provided player and all other players
    /// in the game that they should remove eachother from
    /// their player data list
    fn rem_user_sub(&self, target: &Player) {
        debug!("Removing user subscriptions");

        // Unsubscribe all the clients from eachother
        self.players
            .iter()
            .filter(|other| other.user.id != target.user.id)
            .for_each(|other| {
                target.try_unsubscribe(other.user.id);
                other.try_unsubscribe(target.user.id);
            });
    }
}

/// Attributes map type
pub type AttrMap = TdfMap<String, String>;

pub struct Player {
    pub user: Arc<User>,
    pub link: WeakSessionLink,
    pub notify_handle: SessionNotifyHandle,
    pub net: Arc<NetData>,
    pub state: PlayerState,
    pub attr: AttrMap,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.try_clear_game();
    }
}

impl Player {
    pub fn new(
        user: Arc<User>,
        link: WeakSessionLink,
        notify_handle: SessionNotifyHandle,
        net: Arc<NetData>,
    ) -> Self {
        Self {
            user,
            link,
            notify_handle,
            net,
            state: PlayerState::ActiveConnecting,
            attr: AttrMap::default(),
        }
    }

    pub fn try_clear_game(&self) {
        if let Some(link) = self.link.upgrade() {
            link.clear_game();
        }
    }

    pub fn try_subscribe(&self, user_id: UserId, subscriber: SessionNotifyHandle) {
        if let Some(link) = self.link.upgrade() {
            link.add_subscriber(user_id, subscriber);
        }
    }

    pub fn try_unsubscribe(&self, user_id: UserId) {
        if let Some(link) = self.link.upgrade() {
            link.remove_subscriber(user_id);
        }
    }

    #[inline]
    pub fn notify(&self, packet: Packet) {
        self.notify_handle.notify(packet);
    }

    pub fn encode<S: tdf::TdfSerializer>(&self, game_id: u32, slot: usize, w: &mut S) {
        w.tag_blob_empty(b"BLOB");
        w.tag_owned(b"CONG", self.user.id);
        w.tag_u8(b"CSID", 0);
        w.tag_u8(b"DSUI", 0);
        w.tag_blob_empty(b"EXBL");
        w.tag_owned(b"EXID", self.user.id);
        w.tag_owned(b"GID", game_id);
        w.tag_u8(b"JFPS", 1);
        w.tag_u8(b"JVMM", 1);
        w.tag_u32(b"LOC", 0x64654445);
        w.tag_str(b"NAME", &self.user.username);
        w.tag_str(b"NASP", "cem_ea_id");
        if !self.attr.is_empty() {
            w.tag_ref(b"PATT", &self.attr);
        }
        w.tag_u32(b"PID", self.user.id);
        w.tag_ref(b"PNET", &self.net.addr);

        w.tag_u8(b"PSET", 1);
        w.tag_u8(b"RCRE", 0);
        w.tag_str_empty(b"ROLE");
        w.tag_usize(b"SID", slot);
        w.tag_u8(b"SLOT", 0);
        w.tag_ref(b"STAT", &self.state);
        w.tag_u16(b"TIDX", 0);
        w.tag_u8(b"TIME", 0); /* Unix timestamp in millseconds */
        // User group ID
        w.tag_alt(
            b"UGID",
            ObjectId::new(PLAYER_SESSION_TYPE, self.user.id as u64),
        );

        w.tag_owned(b"UID", self.user.id);

        let uuid = self
            .link
            .upgrade()
            .map(|value| value.uuid.to_string())
            .unwrap_or_default();

        w.tag_str(b"UUID", &uuid);
        w.tag_group_end();
    }
}
