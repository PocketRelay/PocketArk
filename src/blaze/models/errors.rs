use log::error;
use tdf::{TdfSerialize, types::bytes::serialize_bytes};

use crate::{
    blaze::{
        models::{game_manager::GameManagerError, user_sessions::UserSessionsError},
        packet::{FrameFlags, Packet},
        router::IntoPacketResponse,
    },
    database::DbErr,
};

pub type ServerResult<T> = Result<T, BlazeError>;

#[derive(Debug, Clone)]
#[repr(u32)]
#[allow(unused)]
pub enum GlobalError {
    Cancelled = 0x4009,
    Disconnected = 0x4006,
    DuplicateLogin = 0x4007,
    AuthorizationRequired = 0x4008,
    Timeout = 0x4005,
    ComponentNotFound = 0x4002,
    CommandNotFound = 0x4003,
    AuthenticationRequired = 0x4004,
    System = 0x4001,
}

#[derive(Debug, Clone)]
#[repr(u32)]
#[allow(unused)]
pub enum DatabaseError {
    Timeout = 0x406c,
    InitFailure = 0x406d,
    TransactionNotComplete = 0x406e,
    Disconnected = 0x406b,
    NoConnectionAvailable = 0x4068,
    DuplicateEntry = 0x4069,
    System = 0x4065,
}

/// Response type for some blaze error code
pub struct BlazeError(u32);

impl From<DbErr> for BlazeError {
    fn from(value: DbErr) -> Self {
        error!("Database error: {}", value);
        // match value {
        //     DbErr::ConnectionAcquire(_) => DatabaseError::NoConnectionAvailable,
        //     DbErr::Conn(_) => DatabaseError::InitFailure,
        //     _ => DatabaseError::System,
        // }
        // .into()
        DatabaseError::System.into()
    }
}

impl From<GlobalError> for BlazeError {
    fn from(value: GlobalError) -> Self {
        BlazeError(value as u32)
    }
}

impl From<DatabaseError> for BlazeError {
    fn from(value: DatabaseError) -> Self {
        BlazeError(value as u32)
    }
}

struct PreMessage {
    error_code: u32,
}

impl TdfSerialize for PreMessage {
    fn serialize<S: tdf::prelude::TdfSerializer>(&self, w: &mut S) {
        w.tag_u64(b"CNTX", 0);
        w.tag_u32(b"ERRC", self.error_code);
        w.tag_group_empty(b"MADR");
    }
}

impl IntoPacketResponse for BlazeError {
    fn into_response(self, req: &Packet) -> Packet {
        let mut packet = Packet::response_empty(req);
        packet.frame.flags |= FrameFlags::FLAG_NOTIFY;
        // TODO: handle error codes properly
        packet.pre_msg = serialize_bytes(&PreMessage { error_code: self.0 });
        packet
    }
}

impl From<UserSessionsError> for BlazeError {
    fn from(value: UserSessionsError) -> Self {
        BlazeError(value as u32)
    }
}

impl From<GameManagerError> for BlazeError {
    fn from(value: GameManagerError) -> Self {
        BlazeError(value as u32)
    }
}
