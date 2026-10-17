use crate::packet::{FrameFlags, Packet};
use crate::router::IntoPacketResponse;
use tdf::{TdfSerialize, types::bytes::serialize_bytes};

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

/// Response type for some blaze error code
pub struct BlazeError(pub u32);

impl From<GlobalError> for BlazeError {
    fn from(value: GlobalError) -> Self {
        BlazeError(value as u32)
    }
}

pub trait CodedBlazeError {
    fn error_code(self) -> u32;
}

impl<T: CodedBlazeError> From<T> for BlazeError {
    fn from(value: T) -> Self {
        BlazeError(value.error_code())
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
