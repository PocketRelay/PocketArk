use interlink::prelude::Link;

use self::game::manager::GameManager;

pub mod game;

pub struct Services {
    pub games: Link<GameManager>,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();

        Self { games }
    }
}
