use interlink::prelude::Link;

use self::{
    activity::ActivityService, challenges::ChallengesService, character::CharacterService,
    game::manager::GameManager, i18n::I18nService, items::ItemsService,
    match_data::MatchDataService, store::StoreService, strike_teams::StrikeTeamService,
    tokens::Tokens,
};

pub mod activity;
pub mod challenges;
pub mod character;
pub mod game;
pub mod i18n;
pub mod items;
pub mod leaderboard;
pub mod match_data;
pub mod store;
pub mod strike_teams;
pub mod tokens;

pub struct Services {
    pub games: Link<GameManager>,
    pub tokens: Tokens,
    pub match_data: MatchDataService,
    pub challenges: ChallengesService,
    pub activity: ActivityService,
    pub items: ItemsService,
    pub store: StoreService,
    pub character: CharacterService,
    pub i18n: I18nService,
    pub strike_teams: StrikeTeamService,
}

impl Services {
    pub async fn init() -> Self {
        let games = GameManager::start();
        let tokens = Tokens::new().await;
        let match_data = MatchDataService::new();
        let challenges = ChallengesService::new();
        let activity = ActivityService::new();
        let items = ItemsService::new();
        let store = StoreService::new();
        let character = CharacterService::new();
        let i18n = I18nService::new();
        let strike_teams = StrikeTeamService::new();

        Self {
            games,
            tokens,
            match_data,
            challenges,
            activity,
            items,
            store,
            character,
            i18n,
            strike_teams,
        }
    }
}
