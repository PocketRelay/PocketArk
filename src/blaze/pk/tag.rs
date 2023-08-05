//! Implementation for [`Tag`]s and [`TdfType`]s

use super::error::DecodeError;
use std::fmt::{Debug, Display, Write};

/// Represents the tag for a tagged value. Contains the
/// tag itself and the type of value stored after
pub struct Tagged {
    /// The decoded tag
    pub tag: Tag,
    /// The Tdf type after this tag
    pub ty: TdfType,
}

/// Decoded tag bytes type
#[derive(Debug, PartialEq, Eq)]
pub struct Tag(pub [u8; 4]);

impl From<&[u8]> for Tag {
    fn from(value: &[u8]) -> Self {
        let mut out = [0u8; 4];

        // Only copy the max of 4 bytes
        let len = value.len().min(4);
        out[0..len].copy_from_slice(value);

        Self(out)
    }
}

impl From<&[u8; 4]> for Tag {
    fn from(value: &[u8; 4]) -> Self {
        Self(*value)
    }
}

/// Tags are stored as the raw input to avoid extra
/// heap allocation so they must be converted to strings
/// for displaying
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            // Skip empty key bytes
            if byte != 0 {
                f.write_char(byte as char)?;
            }
        }
        Ok(())
    }
}

/// Types from the Blaze packet system which are used to describe
/// what data needs to be decoded.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TdfType {
    /// Variable length integer value
    VarInt = 0x0,
    /// Strings
    String = 0x1,
    /// List of bytes
    Blob = 0x2,
    /// Group of tags
    Group = 0x3,
    /// List of any of the previously mentioned
    List = 0x4,
    /// Map of TdfType to TdfType
    Map = 0x5,
    /// Union of value where with unset type
    Union = 0x6,
    /// List of variable length integers
    VarIntList = 0x7,
    /// Pair of two var int values
    Pair = 0x8,
    /// Three var int values
    Triple = 0x9,
    /// f32 value
    Float = 0xA,
    // Not yet properly decoded
    U12 = 0xC,
}

/// Convert bytes back to tdf types
impl TryFrom<u8> for TdfType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0 => TdfType::VarInt,
            0x1 => TdfType::String,
            0x2 => TdfType::Blob,
            0x3 => TdfType::Group,
            0x4 => TdfType::List,
            0x5 => TdfType::Map,
            0x6 => TdfType::Union,
            0x7 => TdfType::VarIntList,
            0x8 => TdfType::Pair,
            0x9 => TdfType::Triple,
            0xA => TdfType::Float,
            0xC => TdfType::U12,
            ty => return Err(DecodeError::UnknownType { ty }),
        })
    }
}
