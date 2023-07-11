use interlink::{prelude::Link, service::Service};

use crate::blaze::{
    models::{user_sessions::NetData, PlayerState},
    pk::types::TdfMap,
    session::{SessionLink, User},
};

pub mod manager;

pub struct Game {
    /// Unique ID for this game
    pub id: u32,
    /// The current game state
    pub state: u8,
    /// The current game setting
    pub setting: u16,
    /// The game attributes
    pub attributes: AttrMap,
    /// The list of players in this game
    pub players: Vec<Player>,
}

impl Service for Game {
    fn stopping(&mut self) {
        // debug!("Game is stopping (GID: {})", self.id);
        // // Remove the stopping game
        // let services = App::services();
        // let _ = services
        //     .game_manager
        //     .do_send(RemoveGameMessage { game_id: self.id });
    }
}

impl Game {
    pub fn new(id: u32) -> Link<Game> {
        let this = Self {
            id,
            state: 1,
            setting: 262431,
            attributes: [
                ("coopGameVisibility", "1"),
                ("difficulty", "1"),
                ("difficultyRND", ""),
                ("enemytype", "0"),
                ("enemytypeRND", "1"),
                ("level", "0"),
                ("levelRND", "6"),
                ("missionSlot", "0"),
                ("missiontype", "Custom"),
                ("mode", "contact_multiplayer"),
                ("modifierCount", "0"),
                ("modifiers", ""),
            ]
            .into_iter()
            .collect(),
            players: Vec::with_capacity(4),
        };
        this.start()
    }
}

/// Attributes map type
pub type AttrMap = TdfMap<String, String>;

pub struct Player {
    pub user: User,
    pub link: SessionLink,
    pub net: NetData,
    pub state: PlayerState,
    pub attr: AttrMap,
}

impl Player {
    pub fn new(user: User, link: SessionLink, net: NetData) -> Self {
        Self {
            user,
            link,
            net,
            state: PlayerState::ActiveConnecting,
            attr: AttrMap::default(),
        }
    }
}
