use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use chrono::Utc;
use interlink::{
    prelude::{Handler, Link, Message, Sfr},
    service::Service,
};
use log::{debug, error};
use sea_orm::{DatabaseConnection, DbErr};
use tdf::{
    types::string::write_empty_str, ObjectId, TdfDeserialize, TdfMap, TdfSerialize, TdfType,
    TdfTyped,
};
use thiserror::Error;
use uuid::Uuid;

use super::{
    activity::{PrestigeData, PrestigeProgression},
    challenges::CurrencyReward,
    match_data::MatchDataService,
};
use crate::{
    blaze::{
        components::{self, user_sessions::PLAYER_SESSION_TYPE},
        models::{user_sessions::NetworkAddress, PlayerState},
        packet::Packet,
        session::{NetData, SessionLink},
    },
    database::entity::{
        challenge_progress::ProgressUpdateType, users::UserId, ChallengeProgress, Character,
        Currency, InventoryItem, SharedData, User,
    },
    http::models::mission::{
        CompleteMissionData, MissionDetails, MissionModifier, MissionPlayerData, MissionPlayerInfo,
        PlayerInfoBadge, PlayerInfoResult, RewardSource,
    },
    services::{
        activity::{ChallengeStatusChange, ChallengeUpdate},
        game::manager::RemoveGameMessage,
    },
    state::App,
    utils::models::Sku,
};

pub mod manager;

pub type GameID = u32;

pub struct Game {
    /// Unique ID for this game
    pub id: GameID,
    /// The current game state
    pub state: u8,
    /// The current game setting
    pub setting: u32,
    /// The game attributes
    pub attributes: AttrMap,
    /// The list of players in this game
    pub players: Vec<Player>,

    pub modifiers: Vec<MissionModifier>,
    pub mission_data: Option<CompleteMissionData>,
    pub processed_data: Option<MissionDetails>,
}

impl Service for Game {
    fn stopping(&mut self) {
        debug!("Game is stopping (GID: {})", self.id);
        // Remove the stopping game
        let services = App::services();
        let _ = services
            .games
            .do_send(RemoveGameMessage { game_id: self.id });
    }
}

#[derive(Message)]
#[msg(rtype = "Option<MissionDetails>")]
pub struct GetMissionDataMessage(pub DatabaseConnection);

impl Handler<GetMissionDataMessage> for Game {
    type Response = Sfr<Self, GetMissionDataMessage>;
    fn handle(
        &mut self,
        msg: GetMissionDataMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        Sfr::new(move |act: &mut Game, _ctx| {
            Box::pin(async move {
                if let Some(processed) = act.processed_data.clone() {
                    return Some(processed);
                }

                let mission_data = act.mission_data.clone()?;

                let now = Utc::now();
                let db = msg.0;

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

                act.processed_data = Some(data.clone());

                Some(data)
            })
        })
    }
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
    pub items_earned: Vec<InventoryItem>,
    pub challenges_updates: BTreeMap<String, ChallengeUpdate>,
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
            challenges_updates: BTreeMap::new(),
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
        let before = self
            .prestige_progression
            .before
            .get_or_insert(HashMap::new());
        Self::append_prestige(before, shared_data)
    }
    pub fn append_prestige_after(&mut self, shared_data: &SharedData) {
        let after = self
            .prestige_progression
            .after
            .get_or_insert(HashMap::new());
        Self::append_prestige(after, shared_data)
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

    pub fn add_reward_currency(&mut self, name: &str, currency: &str, value: u32) {
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
    db: DatabaseConnection,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    debug!("Processing player data");

    let services = App::services();
    let user = User::get_user(&db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;

    debug!("Loaded processing user");
    let mut shared_data = SharedData::get_from_user(&db, &user).await?;

    debug!("Loaded shared data");

    let mut character = Character::find_by_id_user(&db, &user, shared_data.active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = services
        .character
        .classes
        .by_name(&character.class_name)
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let mut data_builder = PlayerDataBuilder::new();

    debug!("Processing score");

    // Tally up initial base scores awarded from all activities
    data_builder.score = data
        .activity_report
        .activities
        .iter()
        .map(|value| value.attributes.score)
        .sum();

    debug!("Processing badges");

    // Gives awards and badges for each activity
    data.activity_report
        .activities
        .iter()
        .filter_map(|activity| services.match_data.get_by_activity(activity))
        .for_each(|(badge, progress, levels)| {
            let level_name = levels.last().map(|value| value.name.to_string());
            if let Some(level_name) = level_name {
                let badge_name = badge.name.to_string();
                let mut xp_reward: u32 = 0;
                let mut currency_reward = 0;
                let mut level_names = Vec::with_capacity(levels.len());

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
                });
            }
        });

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

    // Update changed challenges
    for (index, challenge_update) in data
        .activity_report
        .activities
        .iter()
        .filter_map(|activity| services.activity.process_activity(activity))
        .enumerate()
    {
        let (model, update_counter, status_change) =
            ChallengeProgress::handle_update(&db, &user, challenge_update).await?;
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

    debug!("Saving character level and xp");

    // Update character level and xp
    if new_xp != previous_xp || level > previous_level {
        character = character.update_xp(&db, new_xp, level).await?
    }

    debug!("Updating currencies");

    // Update currencies
    Currency::create_or_update_many(&db, &user, &data_builder.total_currency).await?;

    let total_currencies_earned = data_builder
        .total_currency
        .into_iter()
        .map(|(name, value)| CurrencyReward { name, value })
        .collect();

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

#[derive(Message)]
pub struct SetModifiersMessage {
    pub modifiers: Vec<MissionModifier>,
}

impl Handler<SetModifiersMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: SetModifiersMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.modifiers = msg.modifiers;
    }
}

#[derive(Message)]
pub struct UpdateStateMessage {
    pub state: u8,
}

impl Handler<UpdateStateMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdateStateMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.state = msg.state;
        self.notify_state();
    }
}

#[derive(Message)]
pub struct RemovePlayerMessage {
    pub user_id: u32,
    pub reason: RemoveReason,
}

impl Handler<RemovePlayerMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: RemovePlayerMessage,
        ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        // Already empty game handling
        if self.players.is_empty() {
            ctx.stop();
            return;
        }

        // Find the player index
        let index = self.players.iter().position(|v| v.user.id == msg.user_id);

        let index = match index {
            Some(value) => value,
            None => return,
        };

        // Remove the player
        let player = self.players.remove(index);

        // Set current game of this player
        player.set_game(None);

        // Update the other players
        self.notify_player_removed(&player, msg.reason);
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
            ctx.stop();
        }
    }
}

#[derive(Message)]
pub struct UpdatePlayerAttr {
    pub attr: AttrMap,
    pub pid: u32,
}

impl Handler<UpdatePlayerAttr> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdatePlayerAttr,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_PLAYER_ATTR_UPDATE,
            NotifyPlayerAttr {
                attr: msg.attr.clone(),
                pid: msg.pid,
                gid: self.id,
            },
        );

        let player = self
            .players
            .iter_mut()
            .find(|player| player.user.id == msg.pid);

        if let Some(player) = player {
            player.attr.insert_presorted(msg.attr.into_inner());
        }
    }
}

#[derive(Message)]
pub struct UpdateGameAttrMessage {
    pub attr: AttrMap,
}

impl Handler<UpdateGameAttrMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdateGameAttrMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_GAME_ATTR_UPDATE,
            NotifyGameAttr {
                attr: msg.attr.clone(),
                gid: self.id,
            },
        );
        self.attributes.insert_presorted(msg.attr.into_inner());
    }
}

#[derive(Message)]
pub struct SetCompleteMissionMessage {
    pub mission_data: CompleteMissionData,
}

impl Handler<SetCompleteMissionMessage> for Game {
    type Response = ();

    fn handle(
        &mut self,
        msg: SetCompleteMissionMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.mission_data = Some(msg.mission_data);
        self.processed_data = None;
    }
}

#[derive(TdfSerialize)]
pub struct NotifyPlayerAttr {
    #[tdf(tag = "ATTR")]
    attr: AttrMap,
    #[tdf(tag = "GID")]
    pid: u32,
    #[tdf(tag = "PID")]
    gid: u32,
}

#[derive(TdfSerialize)]

pub struct NotifyGameAttr {
    #[tdf(tag = "ATTR")]
    attr: AttrMap,
    #[tdf(tag = "GID")]
    gid: u32,
}

impl Game {
    pub fn new(id: u32, attributes: TdfMap<String, String>) -> Link<Game> {
        // TODO: Take attributes provided by client matchmaking
        let this = Self {
            id,
            state: 1,
            setting: 262144,
            attributes,
            players: Vec::with_capacity(4),
            modifiers: Vec::new(),
            mission_data: None,
            processed_data: None,
        };
        this.start()
    }

    /// Notifies all the session and the removed session that a
    /// session was removed from the game.
    ///
    /// `player`    The player that was removed
    /// `player_id` The player ID of the removed player
    fn notify_player_removed(&self, player: &Player, reason: RemoveReason) {
        let packet = Packet::notify(
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_PLAYER_REMOVED,
            PlayerRemoved {
                cntx: 0,
                game_id: self.id,
                player_id: player.user.id,
                reason,
            },
        );
        self.push_all(&packet);
        player.link.push(packet);
    }

    /// Writes the provided packet to all connected sessions.
    /// Does not wait for the write to complete just waits for
    /// it to be placed into each sessions write buffers.
    ///
    /// `packet` The packet to write
    fn push_all(&self, packet: &Packet) {
        self.players
            .iter()
            .for_each(|value| value.link.push(packet.clone()));
    }

    /// Sends a notification packet to all the connected session
    /// with the provided component and contents
    ///
    /// `component` The packet component
    /// `contents`  The packet contents
    fn notify_all<C: TdfSerialize>(&self, component: u16, command: u16, contents: C) {
        let packet = Packet::notify(component, command, contents);
        self.push_all(&packet);
    }

    /// Notifies all players of the current game state
    fn notify_state(&self) {
        self.notify_all(
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_GAME_STATE_UPDATE,
            NotifyStateUpdate {
                game_id: self.id,
                state: self.state,
            },
        );
    }

    /// Creates a subscription between all the users and the the target player
    fn add_user_sub(&self, target_id: UserId, target_link: SessionLink) {
        debug!("Adding user subscriptions");

        // Subscribe all the clients to eachother
        self.players
            .iter()
            .filter(|other| other.user.id.ne(&target_id))
            .for_each(|other| {
                let other_id = other.user.id;
                let other_link = other.link.clone();
                let target_link = target_link.clone();

                tokio::spawn(async move {
                    target_link
                        .add_subscriber(other_id, other_link.clone())
                        .await;
                    other_link
                        .add_subscriber(target_id, target_link.clone())
                        .await;
                });
            });
    }

    /// Notifies the provided player and all other players
    /// in the game that they should remove eachother from
    /// their player data list
    fn rem_user_sub(&self, target_id: UserId, target_link: SessionLink) {
        debug!("Removing user subscriptions");

        // Unsubscribe all the clients from eachother
        self.players
            .iter()
            .filter(|other| other.user.id.ne(&target_id))
            .for_each(|other| {
                let other_id = other.user.id;
                let other_link = other.link.clone();
                let target_link = target_link.clone();

                tokio::spawn(async move {
                    target_link.remove_subscriber(other_id).await;
                    other_link.remove_subscriber(target_id).await;
                });
            });
    }
}

#[derive(TdfSerialize)]
pub struct PlayerRemoved {
    #[tdf(tag = "CNTX")]
    pub cntx: u32,
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "PID")]
    pub player_id: u32,
    #[tdf(tag = "REAS")]
    pub reason: RemoveReason,
}

#[derive(Debug, Clone, Copy, TdfDeserialize, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum RemoveReason {
    /// Hit timeout while joining
    JoinTimeout = 0x0,
    /// Player lost PTP conneciton
    PlayerConnectionLost = 0x1,
    /// Player lost connection with the Pocket Relay server
    ServerConnectionLost = 0x2,
    /// Game migration failed
    MigrationFailed = 0x3,
    GameDestroyed = 0x4,
    GameEnded = 0x5,
    /// Generic player left the game reason
    #[tdf(default)]
    PlayerLeft = 0x6,
    GroupLeft = 0x7,
    /// Player kicked
    PlayerKicked = 0x8,
    /// Player kicked and banned
    PlayerKickedWithBan = 0x9,
    /// Failed to join from the queue
    PlayerJoinFromQueueFailed = 0xA,
    PlayerReservationTimeout = 0xB,
    HostEjected = 0xC,
}

#[derive(Message)]
pub struct NotifyGameReplayMessage;

impl Handler<NotifyGameReplayMessage> for Game {
    type Response = ();

    fn handle(
        &mut self,
        _msg: NotifyGameReplayMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(4, 113, NotifyGameReplay { game_id: self.id })
    }
}

/// Message to add a new player to this game
#[derive(Message)]
pub struct AddPlayerMessage {
    /// The player to add to the game
    pub player: Player,
}

/// Handler for adding a player to the game
impl Handler<AddPlayerMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: AddPlayerMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        let slot = self.players.len();

        self.players.push(msg.player);

        // Obtain the player that was just added
        let player = self
            .players
            .last()
            .expect("Player was added but is missing from players");

        let is_other = slot != 0;
        if is_other {
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
            self.add_user_sub(player.user.id, player.link.clone());
        }

        // Game Setup
        let packet = Packet::notify(
            4,
            20,
            GameDetails {
                game: self,
                player_id: player.user.id,
                // TODO: Type based on why player was added
                ty: MatchmakingResultType::CreatedGame,
            },
        );

        player.link.push(packet);
        player.link.push(Packet::notify(
            4,
            11,
            PostJoinMsg {
                game_id: self.id,
                player_id: player.user.id,
            },
        ));

        // Set current game of this player
        player.set_game(Some(self.id));
    }
}

/// Attributes map type
pub type AttrMap = TdfMap<String, String>;

pub struct Player {
    pub user: Arc<User>,
    pub link: SessionLink,
    pub net: Arc<NetData>,
    pub state: PlayerState,
    pub attr: AttrMap,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.set_game(None);
    }
}

impl Player {
    pub fn new(user: Arc<User>, link: SessionLink, net: Arc<NetData>) -> Self {
        Self {
            user,
            link,
            net,
            state: PlayerState::ActiveConnecting,
            attr: AttrMap::default(),
        }
    }

    pub fn set_game(&self, game: Option<GameID>) {
        let link = self.link.clone();
        tokio::spawn(async move {
            link.set_game(game).await;
        });
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
        w.tag_str(b"UUID", &self.link.uuid.to_string());
        w.tag_group_end();
    }
}

pub struct GameDetails<'a> {
    pub game: &'a Game,
    pub ty: MatchmakingResultType,
    pub player_id: u32,
}

impl<'a> TdfSerialize for GameDetails<'a> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        let game = self.game;
        let host_player = match game.players.first() {
            Some(value) => value,
            None => return,
        };

        // Game details
        w.group(b"GAME", |w| {
            w.tag_list_iter_owned(b"ADMN", game.players.iter().map(|player| player.user.id));
            w.tag_u8(b"APRS", 1);
            w.tag_ref(b"ATTR", &game.attributes);
            w.tag_list_slice(b"CAP", &[4, 0, 0, 0]);
            w.tag_u8(b"CCMD", 3);
            w.tag_str_empty(b"COID");
            w.tag_str_empty(b"CSID");
            w.tag_u64(b"CTIM", 1688851953868334);
            w.group(b"DHST", |w| {
                w.tag_zero(b"CONG");
                w.tag_zero(b"CSID");
                w.tag_zero(b"HPID");
                w.tag_zero(b"HSES");
                w.tag_zero(b"HSLT");
            });
            w.tag_zero(b"DRTO");
            w.group(b"ESID", |w| {
                w.group(b"PS\x20", |w| {
                    w.tag_str_empty(b"NPSI");
                });
                w.group(b"XONE", |w| {
                    w.tag_str_empty(b"COID");
                    w.tag_str_empty(b"ESNM");
                    w.tag_str_empty(b"STMN");
                });
            });

            w.tag_str_empty(b"ESNM");
            w.tag_zero(b"GGTY");

            w.tag_u32(b"GID", game.id);
            w.tag_zero(b"GMRG");
            w.tag_str_empty(b"GNAM");

            w.tag_u64(b"GPVH", 3788120962);
            w.tag_u32(b"GSET", game.setting);
            w.tag_u32(b"GSID", game.id); // SHOULD MATCH START MISSION RESPONSE ID
            w.tag_ref(b"GSTA", &game.state);

            w.tag_str_empty(b"GTYP");
            w.tag_str_empty(b"GURL");
            {
                w.tag_list_start(b"HNET", TdfType::Group, 1);
                w.write_byte(2);

                if let NetworkAddress::AddressPair(addr) = &host_player.net.addr {
                    addr.serialize(w);
                }
            }

            w.tag_u8(b"MCAP", 1); // should be 4?
            w.tag_u8(b"MNCP", 1);
            w.tag_str_empty(b"NPSI");
            // This should be the host QOS details?
            w.group(b"NQOS", |w| {
                w.tag_u32(b"BWHR", 0);
                w.tag_u32(b"DBPS", 24000000);
                w.tag_u32(b"NAHR", 0);
                w.tag_u32(b"NATT", 0); // 1?
                w.tag_u32(b"UBPS", 8000000);
            });

            w.tag_zero(b"NRES");
            w.tag_zero(b"NTOP");
            w.tag_str_empty(b"PGID");
            w.tag_blob_empty(b"PGSR");

            w.group(b"PHST", |w| {
                w.tag_u32(b"CONG", host_player.user.id);
                w.tag_u32(b"CSID", 0);
                w.tag_u32(b"HPID", host_player.user.id);
                w.tag_zero(b"HSLT");
            });
            w.tag_u8(b"PRES", 0x1);
            w.tag_u8(b"PRTO", 0);
            w.tag_str(b"PSAS", "bio-syd");
            w.tag_u8(b"PSEU", 0);
            w.tag_u8(b"QCAP", 0);
            w.group(b"RNFO", |w| {
                w.tag_map_start(b"CRIT", TdfType::String, TdfType::Group, 1);
                write_empty_str(w);
                w.tag_u8(b"RCAP", 1);
                w.tag_group_end();
            });
            w.tag_str_empty(b"SCID");
            w.tag_u32(b"SEED", 131492528);
            w.tag_str_empty(b"STMN");

            w.group(b"THST", |w| {
                w.tag_u32(b"CONG", host_player.user.id);
                w.tag_u8(b"CSID", 0x0);
                w.tag_u32(b"HPID", host_player.user.id);
                w.tag_u32(b"HSES", host_player.user.id);
                w.tag_u8(b"HSLT", 0x0);
            });

            w.tag_list_slice(b"TIDS", &[65534]);
            w.tag_str(b"UUID", "32d89cf8-6a83-4282-b0a0-5b7a8449de2e");
            w.tag_u8(b"VOIP", 0);
            w.tag_str(b"VSTR", "60-Future739583");
        });

        w.tag_u8(b"LFPJ", 0);
        w.tag_str(b"MNAM", "coopGameVisibility");

        // Player list
        w.tag_list_start(b"PROS", TdfType::Group, game.players.len());
        for (slot, player) in game.players.iter().enumerate() {
            player.encode(game.id, slot, w);
        }

        w.group(b"QOSS", |w| {
            w.tag_u8(b"DURA", 0);
            w.tag_u8(b"INTV", 0);
            w.tag_u8(b"SIZE", 0);
        });
        w.tag_u8(b"QOSV", 0);

        w.tag_union_start(b"REAS", 0x3);
        w.group(b"MMSC", |w| {
            const FIT: u16 = 20000; // 24500

            w.tag_u16(b"FIT", FIT);
            w.tag_u16(b"FIT", 0);
            w.tag_u16(b"MAXF", FIT);
            w.tag_u32(b"MSCD", self.player_id);
            w.tag_u32(b"MSID", self.player_id);

            // TODO: Matchmaking result
            // SUCCESS_CREATED_GAME = 0
            // SUCCESS_JOINED_NEW_GAME = 1
            // SUCCESS_JOINED_EXISTING_GAME = 2
            // SESSION_TIMED_OUT = 3
            // SESSION_CANCELED = 4
            // SESSION_TERMINATED = 5
            // SESSION_ERROR_GAME_SETUP_FAILED = 6
            w.tag_owned(b"RSLT", self.ty);

            w.tag_u32(b"TOUT", 15000000);
            w.tag_u32(b"TTM", 51109);

            w.tag_u32(b"USID", self.player_id);
        });
    }
}

#[derive(Debug, Clone, Copy, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum MatchmakingResultType {
    CreatedGame = 0,
    JoinedNewGame = 1,
    JoinedExistingGame = 2,
}

pub struct PostJoinMsg {
    pub player_id: u32,
    pub game_id: u32,
}

impl TdfSerialize for PostJoinMsg {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group(b"CONV", |w| {
            w.tag_zero(b"FCNT");
            w.tag_zero(b"NTOP");
            w.tag_zero(b"TIER");
        });
        w.tag_u8(b"DISP", 1);
        w.tag_owned(b"GID", self.game_id);

        w.tag_alt(b"GRID", ObjectId::new_raw(0, 0, 0));

        w.tag_owned(b"MSCD", self.player_id);
        w.tag_owned(b"MSID", self.player_id);
        w.tag_zero(b"QSVR");
        w.tag_owned(b"USID", self.player_id);
    }
}

#[derive(TdfSerialize)]
struct NotifyStateUpdate {
    #[tdf(tag = "GID")]
    game_id: u32,
    #[tdf(tag = "GSTA")]
    state: u8,
}

struct NotifyGameReplay {
    game_id: u32,
}

impl TdfSerialize for NotifyGameReplay {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_owned(b"GID", self.game_id);
        w.tag_owned(b"GRID", self.game_id)
    }
}
