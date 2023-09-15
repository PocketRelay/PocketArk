use log::{debug, warn};
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::RwLock;

use crate::{
    blaze::models::{
        game_manager::{GameSetupContext, MatchmakingResult},
        PlayerState,
    },
    utils::hashing::IntHashMap,
};

use super::{AttrMap, Game, GameID, GameRef, Player, DEFAULT_FIT};

/// Manager which controls all the active games on the server
/// commanding them to do different actions and removing them
/// once they are no longer used
pub struct GameManager {
    /// The map of games to the actual game address
    games: RwLock<IntHashMap<GameID, GameRef>>,
    /// Stored value for the ID to give the next game
    next_id: AtomicU32,
}

impl GameManager {
    /// Max number of times to poll a game for shutdown before erroring
    const MAX_RELEASE_ATTEMPTS: u8 = 5;

    /// Starts a new game manager service returning its link
    pub fn new() -> Self {
        Self {
            games: Default::default(),
            next_id: AtomicU32::new(1),
        }
    }

    pub async fn create(&self, mut host: Player, attributes: AttrMap) -> (GameRef, GameID) {
        let games = &mut *self.games.write().await;

        let id = self.next_id.fetch_add(1, Ordering::AcqRel);

        host.state = PlayerState::ActiveConnected;

        let game = Arc::new(RwLock::new(Game::new(id, attributes)));
        games.insert(id, game.clone());

        let link = game.clone();

        tokio::spawn(async move {
            let context = GameSetupContext::Matchmaking {
                fit_score: DEFAULT_FIT,
                max_fit_score: DEFAULT_FIT,
                id_1: host.user.id,
                id_2: host.user.id,
                result: MatchmakingResult::CreatedGame,
                tout: 15000000,
                ttm: 51109,
                id_3: host.user.id,
            };

            // TODO: Aquire lock outside of future? to prevent game usage before the games write lock is dropped
            let link = &mut *link.write().await;
            link.add_player(host, context);
        });

        (game, id)
    }

    pub async fn get_game(&self, game_id: GameID) -> Option<GameRef> {
        let games = &*self.games.read().await;
        games.get(&game_id).cloned()
    }

    pub async fn remove_game(&self, game_id: GameID) {
        let games = &mut *self.games.write().await;
        if let Some(mut game) = games.remove(&game_id) {
            let mut attempt: u8 = 1;

            // Attempt to obtain the owned game
            let game = loop {
                if attempt > Self::MAX_RELEASE_ATTEMPTS {
                    let references = Arc::strong_count(&game);
                    warn!(
                        "Failed to stop game {} there are still {} references to it",
                        game_id, references
                    );
                    return;
                }

                match Arc::try_unwrap(game) {
                    Ok(value) => break value,
                    Err(arc) => {
                        let wait = 5 * attempt as u64;
                        let references = Arc::strong_count(&arc);
                        debug!(
                            "Game {} still has {} references to it, waiting {}s",
                            game_id, references, wait
                        );
                        tokio::time::sleep(Duration::from_secs(wait)).await;
                        game = arc;
                        attempt += 1;
                        continue;
                    }
                }
            };

            let game = game.into_inner();
            game.stopped();
        }
    }
}
