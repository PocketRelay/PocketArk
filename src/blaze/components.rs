/// Key created from a component and command
pub type ComponentKey = u32;

/// Creates an u32 value from the provided component
/// and command merging them into a single u32
pub const fn component_key(component: u16, command: u16) -> ComponentKey {
    ((component as u32) << 16) + command as u32
}

pub mod authentication {
    pub const COMPONENT: u16 = 1; // 0x0001

    pub const AUTHENTICATE: u16 = 10; // 0x000a
    pub const LIST_ENTITLEMENTS_2: u16 = 29; // 0x001d
}

// Missing packet handler for 0x0004->0x0041 (4->65) PACKER_CREATE_GAME
// Missing packet handler for 0x0004->0x0043 (4->67) MESH_ENDPOINTS_CONNECTION_LOST

pub mod game_manager {
    use tdf::ObjectType;

    pub const COMPONENT: u16 = 4; // 0x0004

    pub const UPDATE_GAME_STATE: u16 = 3;
    pub const UPDATE_GAME_ATTR: u16 = 7; // 0x0007
    pub const UPDATE_PLAYER_ATTR: u16 = 8; // 0x0008
    pub const FINALIZE_GAME_CREATION: u16 = 15;
    pub const START_MATCHMAKING_SCENARIO: u16 = 16; // 0x0010
    pub const CANCEL_MATCHMAKING_SCENARIO: u16 = 17; // 0x0011
    pub const REPLAY_GAME: u16 = 19; // 0x0013
    pub const LEAVE_GAME_BY_GROUP: u16 = 22; // 0x0016
    pub const ADD_ADMIN_PLAYER: u16 = 106; // 0x6A
    pub const SET_GAME_SETTINGS: u16 = 4; // 0x4
    pub const REPORT_TELEMETRY: u16 = 171; // 0x00ab

    // Notifications
    pub const MATCHMAKING_SESSION_CANCELLED: u16 = 10; // 0x000a
    pub const MATCHMAKING_SESSION_CONNECTION_VALIDATED: u16 = 11; // 0x000b
    pub const MATCHMAKING_ASYNC_STATUS: u16 = 12; // 0x000c
    pub const GAME_SETUP: u16 = 20; // 0x0014
    pub const PLAYER_JOINING: u16 = 21; // 0x0015
    pub const PLAYER_JOIN_COMPLETED: u16 = 30; // 0x001E
    pub const PLAYER_REMOVED: u16 = 40;

    pub const MESH_ENDPOINTS_CONNECTED: u16 = 65; // 0x0041
    pub const MESH_ENDPOINTS_CONNECTION_LOST: u16 = 67;

    pub const GAME_ATTR_UPDATE: u16 = 80; // 0x0050
    pub const PLAYER_ATTR_UPDATE: u16 = 90; // 0x005a
    pub const GAME_STATE_CHANGE: u16 = 100; // 0x0064
    pub const GAME_SETTINGS_CHANGE: u16 = 110; // 0x006e

    pub const GAME_REPORTING_ID_CHANGE: u16 = 113; // 0x0071

    pub const GAME_PLAYER_STATE_CHANGE: u16 = 116; // 0x0074

    pub const ADMIN_LIST_CHANGE: u16 = 202; // 0x00ca

    pub const GAME_TYPE: ObjectType = ObjectType::new(COMPONENT, 1);
}

pub mod util {
    pub const COMPONENT: u16 = 9; // 0x0009

    pub const FETCH_CLIENT_CONFIG: u16 = 1; // 0x0001
    pub const PING: u16 = 2; // 0x0002

    pub const PRE_AUTH: u16 = 7; // 0x0007
    pub const POST_AUTH: u16 = 8; // 0x0008
    pub const SET_CLIENT_METRICS: u16 = 22;
}

pub mod user_sessions {
    use tdf::ObjectType;

    pub const COMPONENT: u16 = 30722; // 0x7802

    pub const UPDATE_HARDWARE_FLAGS: u16 = 8; // 0x0008
    pub const LOOKUP_USER: u16 = 12; // 0x000c

    pub const UPDATE_NETWORK_INFO: u16 = 20; // 0x0014

    pub const USER_SESSION_EXTENDED_DATA_UPDATE: u16 = 1; // 0x0001
    pub const USER_ADDED: u16 = 2; // 0x0002
    pub const USER_REMOVED: u16 = 3; // 0x0003
    pub const USER_UPDATED: u16 = 5; //0x0005
    pub const UPDATE_AUTH: u16 = 8; // 0x0008

    pub const PLAYER_SESSION_TYPE: ObjectType = ObjectType::new(COMPONENT, 2);
}
