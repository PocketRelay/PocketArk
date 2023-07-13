use interlink::prelude::Link;

use self::{defs::Definitions, game::manager::GameManager};

pub mod defs;
pub mod game;

pub struct Services {
    pub games: Link<GameManager>,
    pub defs: Definitions,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let defs = Definitions::load();
        Self { games, defs }
    }
}
