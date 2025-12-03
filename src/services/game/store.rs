use crate::{
    services::game::{Game, GameID, GameRef},
    utils::hashing::IntHashMap,
};
use parking_lot::RwLock;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

pub struct Games {
    /// Stored value for the ID to give the next game
    next_id: AtomicU32,
    /// The map of games to the actual game address
    games: RwLock<IntHashMap<GameID, GameRef>>,
}

impl Default for Games {
    fn default() -> Self {
        Self {
            next_id: AtomicU32::new(1),
            games: Default::default(),
        }
    }
}

impl Games {
    /// Obtains the total count of games in the list
    pub fn total(&self) -> usize {
        self.games.read().len()
    }

    pub fn remove_by_id(&self, game_id: GameID) {
        _ = self.games.write().remove(&game_id);
    }

    pub fn get_by_id(&self, game_id: GameID) -> Option<GameRef> {
        self.games.read().get(&game_id).cloned()
    }

    // Get the next available game ID
    pub fn next_id(&self) -> GameID {
        self.next_id.fetch_add(1, Ordering::AcqRel)
    }

    pub fn insert(&self, game: Game) -> GameRef {
        let game_id = game.id;
        let link = Arc::new(RwLock::new(game));
        self.games.write().insert(game_id, link.clone());
        link
    }
}
