use interlink::prelude::Link;

use self::{
    activity::ActivityService, challenges::ChallengesService, defs::Definitions,
    game::manager::GameManager, items::ItemsService, match_data::MatchDataService,
    store::StoreService, tokens::Tokens,
};

pub mod activity;
pub mod challenges;
pub mod defs;
pub mod game;
pub mod items;
pub mod match_data;
pub mod store;
pub mod tokens;

pub struct Services {
    pub games: Link<GameManager>,
    pub defs: Definitions,
    pub tokens: Tokens,
    pub match_data: MatchDataService,
    pub challenges: ChallengesService,
    pub activity: ActivityService,
    pub items: ItemsService,
    pub store: StoreService,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let defs = Definitions::load();
        let tokens = Tokens::new().await;
        let match_data = MatchDataService::load();
        let challenges = ChallengesService::load();
        let activity = ActivityService::new();
        let items = ItemsService::load();
        let store = StoreService::load();

        Self {
            games,
            defs,
            tokens,
            match_data,
            challenges,
            activity,
            items,
            store,
        }
    }
}
