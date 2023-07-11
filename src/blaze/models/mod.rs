use serde::Serialize;

use crate::blaze::pk::{
    codec::{Decodable, Encodable},
    error::DecodeResult,
    reader::TdfReader,
    writer::TdfWriter,
};

use super::pk::{codec::ValueType, tag::TdfType};

pub mod auth;
pub mod game_manager;
pub mod user_sessions;
pub mod util;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerState {
    /// Link between the mesh points is not connected
    Reserved = 0x0,
    Queued = 0x1,
    /// Link is being formed between two mesh points
    ActiveConnecting = 0x2,
    ActiveMigrating = 0x3,
    /// Link is connected between two mesh points
    ActiveConnected = 0x4,
    ActiveKickPending = 0x5,
}

impl PlayerState {
    /// Gets the mesh state from the provided value
    ///
    /// `value` The value of the mesh state
    pub fn from_value(value: u8) -> Self {
        match value {
            0x0 => Self::Reserved,
            0x1 => Self::Queued,
            0x2 => Self::ActiveConnecting,
            0x3 => Self::ActiveMigrating,
            0x4 => Self::ActiveConnected,
            0x5 => Self::ActiveKickPending,
            _ => Self::Reserved,
        }
    }
}

impl Encodable for PlayerState {
    fn encode(&self, output: &mut TdfWriter) {
        output.write_u8((*self) as u8)
    }
}

impl Decodable for PlayerState {
    fn decode(reader: &mut TdfReader) -> DecodeResult<Self> {
        Ok(PlayerState::from_value(reader.read_u8()?))
    }
}

impl ValueType for PlayerState {
    fn value_type() -> TdfType {
        TdfType::VarInt
    }
}
