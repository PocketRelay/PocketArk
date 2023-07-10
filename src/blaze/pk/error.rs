//! Error type used when decoding packets [`DecodeError`] and result
//! type alias [`DecodeResult`]

use super::tag::{Tag, TdfType};
use std::{error::Error, fmt::Display};

/// Error type for errors that can occur while decoding a value
/// using the tdf decode
#[derive(Debug)]
pub enum DecodeError {
    /// The tag that was expected could not be found
    MissingTag {
        /// The tag that was being searched for
        tag: Tag,
        /// The type of the tag being searched for
        ty: TdfType,
    },
    /// The found tag was not of the correct type
    InvalidTagType {
        /// The tag which the type was invalid for
        tag: Tag,
        /// The expected tdf type
        expected: TdfType,
        /// The actual tdf type
        actual: TdfType,
    },
    /// Encountered an unexpected type when decoding a
    /// map or list
    InvalidType {
        /// The expected tdf type
        expected: TdfType,
        /// The actual tdf type
        actual: TdfType,
    },

    /// Encountered an unknown tag type
    UnknownType {
        /// The tag type value
        ty: u8,
    },

    /// Reached the end of the available bytes before
    /// a value could be obtained
    UnexpectedEof {
        /// The current reader cusor position
        cursor: usize,
        /// The number of bytes attempted to read
        wanted: usize,
        /// The remaining bytes in the reader slice
        remaining: usize,
    },

    /// Other error type with custom message
    Other(&'static str),
}

/// Type alias for result which could result in a Decode Error
pub type DecodeResult<T> = Result<T, DecodeError>;

/// Error implementation
impl Error for DecodeError {}

/// Display formatting implementation
impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::MissingTag { tag, ty } => {
                write!(f, "Missing tag '{}' (type: {:?})", tag, ty)
            }
            DecodeError::InvalidTagType {
                tag,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Invalid tag type for '{}' (expected: {:?}, got: {:?})",
                    tag, expected, actual
                )
            }
            DecodeError::InvalidType { expected, actual } => {
                write!(
                    f,
                    "Unexpected tag type (expected: {:?}, got: {:?})",
                    expected, actual
                )
            }
            DecodeError::UnknownType { ty } => {
                write!(f, "Unknown tag type: {}", ty)
            }
            DecodeError::UnexpectedEof {
                cursor,
                wanted,
                remaining,
            } => {
                write!(
                    f,
                    "Unexpected end of file (cursor: {}, wanted: {}, remaining: {})",
                    cursor, wanted, remaining
                )
            }
            DecodeError::Other(err) => f.write_str(err),
        }
    }
}
