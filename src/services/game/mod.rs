use crate::{
    blaze::{
        components::{self, game_manager},
        models::game_manager::{
            AttributesChange, GameSetupContext, GameSetupResponse, JoinComplete, NotifyGameReplay,
            NotifyGameStateChange, NotifyPostJoinedGame, PlayerAttributesChange,
            PlayerNetConnectionStatus, PlayerRemoved, PlayerState, PlayerStateChange, RemoveReason,
        },
        packet::Packet,
        session::SessionLink,
    },
    config::Config,
    database::entity::users::UserId,
    http::models::mission::{CompleteMissionData, MissionDetails, MissionModifier},
    services::{
        game::{player::GamePlayer, store::Games},
        tunnel::TunnelService,
    },
};
use log::{debug, warn};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use tdf::TdfMap;

pub mod data;
pub mod player;
pub mod store;

/// Attributes map type
pub type AttrMap = TdfMap<String, String>;

pub type GameID = u32;
pub type GameRef = Arc<RwLock<Game>>;
pub type WeakGameRef = Weak<RwLock<Game>>;

pub trait GameAddPlayerExt {
    fn add_player(
        &self,
        tunnel_service: &TunnelService,
        config: &Config,
        player: GamePlayer,
        session: SessionLink,
        context: GameSetupContext,
    );
}

impl GameAddPlayerExt for GameRef {
    fn add_player(
        &self,
        tunnel_service: &TunnelService,
        config: &Config,
        player: GamePlayer,
        session: SessionLink,
        context: GameSetupContext,
    ) {
        // Add the player to the game
        let (game_id, index) = {
            let game = &mut *self.write();
            let slot = game.add_player(player, context, config);
            (game.id, slot)
        };

        // Allocate tunnel if supported by client
        if let Some(association) = session.data.get_association() {
            tunnel_service.associate_pool(association, game_id, index as u8);
        }

        // Update the player current game
        session.data.set_game(game_id, Arc::downgrade(self));
    }
}

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
    pub players: Vec<GamePlayer>,

    pub modifiers: Vec<MissionModifier>,
    pub mission_data: Option<CompleteMissionData>,
    pub processed_data: Option<MissionDetails>,

    /// Services access
    pub games_store: Arc<Games>,
    /// Access to the tunneling service
    pub tunnel_service: Arc<TunnelService>,
}

pub const DEFAULT_FIT: u16 = 21600;

impl Game {
    pub const MAX_PLAYERS: usize = 4;

    pub fn new(
        id: u32,
        attributes: TdfMap<String, String>,
        games_store: Arc<Games>,
        tunnel_service: Arc<TunnelService>,
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
            games_store,
            tunnel_service,
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

    /// Get the mission data for processing
    pub fn get_mission_data(&self) -> Option<CompleteMissionData> {
        self.mission_data.clone()
    }

    pub fn get_processed_data(&self) -> Option<MissionDetails> {
        self.processed_data.clone()
    }

    pub fn set_processed(&mut self, data: MissionDetails) {
        self.processed_data = Some(data);
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

    fn stop(&mut self) {
        // Mark the game as stopping
        // self.state = GameState::Destructing;

        if !self.players.is_empty() {
            warn!("Game {} was stopped with players still present", self.id);
        }

        // Remove the stopping game
        self.games_store.remove_by_id(self.id);
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

        // Remove the tunnel
        self.tunnel_service.dissociate_pool(self.id, index as u8);

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

        drop(player);

        // If the player was in the host slot attempt migration
        if index == 0 {
            // self.try_migrate_host();
        }

        if self.players.is_empty() {
            // Game is empty stop it
            self.stop();
        }
    }

    pub fn add_player(
        &mut self,
        player: GamePlayer,
        context: GameSetupContext,
        config: &Config,
    ) -> usize {
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
                config,
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

    #[allow(unused)]
    pub fn update_mesh(&mut self, target_id: UserId, status: PlayerNetConnectionStatus) {
        // We only care about a connected state
        if !matches!(status, PlayerNetConnectionStatus::Connected) {
            return;
        }

        // Obtain the target player
        let target_slot = match self
            .players
            .iter_mut()
            .find(|slot| slot.user.id == target_id)
        {
            Some(value) => value,
            None => {
                debug!(
                    "Unable to find player to update mesh state for (PID: {} GID: {})",
                    target_id, self.id
                );
                return;
            }
        };

        // Mark the player as connected and update the state for all users
        target_slot.state = PlayerState::ActiveConnected;

        self.notify_all(Packet::notify(
            game_manager::COMPONENT,
            game_manager::GAME_PLAYER_STATE_CHANGE,
            PlayerStateChange {
                gid: self.id,
                pid: target_id,
                state: PlayerState::ActiveConnected,
            },
        ));

        // Notify all players that the player has completely joined
        self.notify_all(Packet::notify(
            game_manager::COMPONENT,
            game_manager::PLAYER_JOIN_COMPLETED,
            JoinComplete {
                game_id: self.id,
                player_id: target_id,
            },
        ));
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
    fn notify_player_removed(&self, player: &GamePlayer, reason: RemoveReason) {
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

        // TODO: Is this supposed to be here?
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
    fn add_user_sub(&self, target: &GamePlayer) {
        debug!("Adding user subscriptions");

        // Subscribe all the clients to each other
        self.players
            .iter()
            .filter(|other| other.user.id != target.user.id)
            .for_each(|other| {
                target.try_subscribe(other.user.id, other.link.clone());
                other.try_subscribe(target.user.id, target.link.clone());
            });
    }

    /// Notifies the provided player and all other players
    /// in the game that they should remove each other from
    /// their player data list
    fn rem_user_sub(&self, target: &GamePlayer) {
        debug!("Removing user subscriptions");

        // Unsubscribe all the clients from each other
        self.players
            .iter()
            .filter(|other| other.user.id != target.user.id)
            .for_each(|other| {
                target.try_unsubscribe(other.user.id);
                other.try_unsubscribe(target.user.id);
            });
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        debug!("Game is stopped (GID: {})", self.id);
    }
}
