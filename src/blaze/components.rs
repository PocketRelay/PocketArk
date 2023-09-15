/// Key created from a component and command
pub type ComponentKey = u32;

/// Creates an u32 value from the provided component
/// and command merging them into a single u32
pub const fn component_key(component: u16, command: u16) -> ComponentKey {
    ((component as u32) << 16) + command as u32
}

pub mod authentication {
    pub const COMPONENT: u16 = 1;

    pub const AUTHENTICATE: u16 = 10;
    pub const LIST_ENTITLEMENTS_2: u16 = 29;
}

pub mod game_manager {
    use tdf::ObjectType;

    pub const COMPONENT: u16 = 4;

    pub const UPDATE_GAME_STATE: u16 = 3;
    pub const UPDATE_GAME_ATTR: u16 = 7;
    pub const UPDATE_PLAYER_ATTR: u16 = 8;
    pub const START_MATCHMAKING: u16 = 16;
    pub const REPLAY_GAME: u16 = 19;
    pub const LEAVE_GAME_BY_GROUP: u16 = 22;

    pub const NOTIFY_PLAYER_REMOVED: u16 = 40;
    pub const NOTIFY_GAME_ATTR_UPDATE: u16 = 80;
    pub const NOTIFY_PLAYER_ATTR_UPDATE: u16 = 90;
    pub const NOTIFY_GAME_STATE_UPDATE: u16 = 100;

    pub const GAME_TYPE: ObjectType = ObjectType::new(COMPONENT, 1);
}

pub mod util {
    pub const COMPONENT: u16 = 9;

    pub const FETCH_CLIENT_CONFIG: u16 = 1;
    pub const PING: u16 = 2;
    pub const PRE_AUTH: u16 = 7;
    pub const POST_AUTH: u16 = 8;
}

pub mod user_sessions {
    use tdf::ObjectType;

    pub const COMPONENT: u16 = 30722;

    pub const UPDATE_HARDWARE_FLAGS: u16 = 8;
    pub const UPDATE_NETWORK_INFO: u16 = 20;

    pub const USER_SESSION_EXTENDED_DATA_UPDATE: u16 = 1;
    pub const USER_ADDED: u16 = 2;
    pub const USER_REMOVED: u16 = 3;
    pub const UPDATE_AUTH: u16 = 8;

    pub const PLAYER_SESSION_TYPE: ObjectType = ObjectType::new(COMPONENT, 2);
}
