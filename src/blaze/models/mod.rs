use blaze_pk::{
    codec::{Decodable, Encodable},
    error::DecodeResult,
    reader::TdfReader,
    writer::TdfWriter,
};

pub mod auth;
pub mod util;

pub struct UpdateNetworkInfo {}

pub struct EmptyData;

impl Encodable for EmptyData {
    fn encode(&self, writer: &mut TdfWriter) {}
}

impl Decodable for EmptyData {
    fn decode(reader: &mut TdfReader) -> DecodeResult<Self> {
        Ok(EmptyData)
    }
}
