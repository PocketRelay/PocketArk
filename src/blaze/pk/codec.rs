//! Traits for implementing encoding ([`Encodable`]) and decoding ([`Decodable`])
//! for different types and [`ValueType`] trait for specifying the Tdf type of a type

use super::{error::DecodeResult, reader::TdfReader, tag::TdfType, writer::TdfWriter};

/// Trait for something that can be decoded from a TdfReader
pub trait Decodable: Sized {
    /// Function for implementing decoding of Self from
    /// the provided Reader. Will return None if self
    /// cannot be decoded
    ///
    /// `reader` The reader to decode from
    fn decode(r: &mut TdfReader) -> DecodeResult<Self>;
}

/// Trait for something that can be encoded onto a TdfWriter
pub trait Encodable: Sized {
    /// Function for implementing encoding of Self to the
    /// provided vec of bytes
    ///
    /// `writer` The output to encode to
    fn encode(&self, w: &mut TdfWriter);

    /// Shortcut function for encoding self directly to
    /// a Vec of bytes
    fn encode_bytes(&self) -> Vec<u8> {
        let mut output = TdfWriter::default();
        self.encode(&mut output);
        output.into()
    }
}

/// Trait for a type that conforms to one of the standard TdfTypes
/// used on structures that implement Decodable or Encodable to allow
/// them to be encoded as tag fields
pub trait ValueType {
    /// The type of tdf value this is
    fn value_type() -> TdfType;
}

/// Macro for generating the ValueType implementation for a type
#[macro_export]
macro_rules! value_type {
    ($for:ty, $type:expr) => {
        impl $crate::blaze::pk::codec::ValueType for $for {
            fn value_type() -> $crate::blaze::pk::tag::TdfType {
                $type
            }
        }
    };
}
