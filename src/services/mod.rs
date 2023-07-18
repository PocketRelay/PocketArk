use interlink::prelude::Link;

use self::{
    defs::Definitions, game::manager::GameManager, match_data::MatchDataService, tokens::Tokens,
};

pub mod defs;
pub mod game;
pub mod items;
pub mod match_data;
pub mod tokens;

pub struct Services {
    pub games: Link<GameManager>,
    pub defs: Definitions,
    pub tokens: Tokens,
    pub match_data: MatchDataService,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let defs = Definitions::load();
        let tokens = Tokens::new().await;
        let badges = MatchDataService::load();
        Self {
            games,
            defs,
            tokens,
            match_data: badges,
        }
    }
}
