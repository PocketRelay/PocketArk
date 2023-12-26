pub mod constants;
pub mod hashing;
pub mod lock;
pub mod logging;
pub mod models;
pub mod signing;

/// Type alias for an immutable string without its capacity
pub type ImStr = Box<str>;

/// Asserts the provided `condition` is true, returning the
/// provided `error` if its false
#[inline]
pub fn require<E>(condition: bool, error: E) -> Result<(), E> {
    match condition {
        true => Ok(()),
        false => Err(error),
    }
}

/// Asserts the provided `condition` is false, returning the
/// provided `error` if its true
#[inline]
pub fn require_not<E>(condition: bool, error: E) -> Result<(), E> {
    match condition {
        false => Ok(()),
        true => Err(error),
    }
}
