use self::{
    challenges::ChallengesService, character::CharacterService, i18n::I18nService,
    items::ItemsService, match_data::MatchDataService, store::StoreService,
    strike_teams::StrikeTeamService,
};

pub mod activity;
pub mod challenges;
pub mod character;
pub mod game;
pub mod i18n;
pub mod items;
pub mod match_data;
pub mod sessions;
pub mod store;
pub mod strike_teams;

/// Static storage for the services structure when initalized
static mut SERVICES: Option<Services> = None;

pub struct Services {
    pub match_data: MatchDataService,
    pub challenges: ChallengesService,
    pub items: ItemsService,
    pub store: StoreService,
    pub character: CharacterService,
    pub i18n: I18nService,
    pub strike_teams: StrikeTeamService,
}

impl Services {
    pub fn init_global() {
        let value = Self::init().unwrap();

        unsafe { SERVICES = Some(value) };
    }

    pub fn get() -> &'static Services {
        match unsafe { &SERVICES } {
            Some(value) => value,
            None => panic!("Global services not initialized"),
        }
    }

    pub fn init() -> anyhow::Result<Self> {
        let match_data = MatchDataService::new()?;
        let challenges = ChallengesService::new()?;
        let items = ItemsService::new()?;
        let store = StoreService::new()?;
        let character = CharacterService::new()?;
        let i18n = I18nService::new();
        let strike_teams = StrikeTeamService::new()?;

        Ok(Self {
            match_data,
            challenges,
            items,
            store,
            character,
            i18n,
            strike_teams,
        })
    }
}
