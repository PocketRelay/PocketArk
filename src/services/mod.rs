use interlink::prelude::Link;

use self::{
    activity::ActivityService, challenges::ChallengesService, defs::Definitions,
    game::manager::GameManager, match_data::MatchDataService, tokens::Tokens,
};

pub mod activity;
pub mod challenges;
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
    pub challenges: ChallengesService,
    pub activity: ActivityService,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let defs = Definitions::load();
        let tokens = Tokens::new().await;
        let match_data = MatchDataService::load();
        let challenges = ChallengesService::load();
        let activity = ActivityService::new();
        Self {
            games,
            defs,
            tokens,
            match_data,
            challenges,
            activity,
        }
    }
}
