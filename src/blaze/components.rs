pub mod authentication {
    pub const COMPONENT: u16 = 1;

    pub const AUTHENTICATE: u16 = 10;
}

pub mod game_manager {
    pub const COMPONENT: u16 = 4;
}
pub mod util {
    pub const COMPONENT: u16 = 9;

    pub const FETCH_CLIENT_CONFIG: u16 = 1;
    pub const PING: u16 = 2;
    pub const PRE_AUTH: u16 = 7;
    pub const POST_AUTH: u16 = 8;
}

pub mod user_sessions {
    pub const COMPONENT: u16 = 30722;

    pub const UPDATE_AUTH: u16 = 8;
    pub const UPDATE_NETWORK_INFO: u16 = 20;
}
