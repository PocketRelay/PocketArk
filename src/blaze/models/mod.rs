use serde::Serialize;
use tdf::{TdfDeserialize, TdfSerialize, TdfTyped};

pub mod auth;
pub mod game_manager;
pub mod user_sessions;
pub mod util;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, TdfDeserialize, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum PlayerState {
    /// Link between the mesh points is not connected
    #[tdf(default)]
    Reserved = 0x0,
    Queued = 0x1,
    /// Link is being formed between two mesh points
    ActiveConnecting = 0x2,
    ActiveMigrating = 0x3,
    /// Link is connected between two mesh points
    ActiveConnected = 0x4,
    ActiveKickPending = 0x5,
}
