use interlink::prelude::Link;

use self::{defs::Definitions, game::manager::GameManager, tokens::Tokens};

pub mod defs;
pub mod game;
pub mod tokens;

pub struct Services {
    pub games: Link<GameManager>,
    pub defs: Definitions,
    pub tokens: Tokens,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let defs = Definitions::load();
        let tokens = Tokens::new().await;
        Self {
            games,
            defs,
            tokens,
        }
    }
}
