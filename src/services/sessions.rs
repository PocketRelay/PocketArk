//! Service for storing links to all the currenly active
//! authenticated sessions on the server

use crate::blaze::session::{SessionLink, WeakSessionLink};
use crate::database::entity::users::UserId;
use crate::http::models::HttpError;
use crate::utils::hashing::IntHashMap;
use crate::utils::signing::SigningKey;
use base64ct::{Base64UrlUnpadded, Encoding};
use hyper::StatusCode;
use parking_lot::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

type SessionMap = IntHashMap<UserId, WeakSessionLink>;

/// Service for storing links to authenticated sessions and
/// functionality for authenticating sessions
pub struct Sessions {
    /// Lookup mapping between User IDs and their session links
    ///
    /// This uses a blocking mutex as there is little to no overhead
    /// since all operations are just map read and writes which don't
    /// warrant the need for the async variant
    sessions: Mutex<SessionMap>,

    /// HMAC key used for computing signatures
    key: SigningKey,
}

impl Sessions {
    /// Expiry time for tokens
    const EXPIRY_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 30 /* 30 Days */);

    /// Starts a new service returning its link
    pub fn new(key: SigningKey) -> Self {
        Self {
            sessions: Default::default(),
            key,
        }
    }

    pub fn create_token(&self, user_id: UserId) -> String {
        // Compute expiry timestamp
        let exp = SystemTime::now()
            .checked_add(Self::EXPIRY_TIME)
            .expect("Expiry timestamp too far into the future")
            .duration_since(UNIX_EPOCH)
            .expect("Clock went backwards")
            .as_secs();

        // Create encoded token value
        let mut data = [0u8; 12];
        data[..4].copy_from_slice(&user_id.to_be_bytes());
        data[4..].copy_from_slice(&exp.to_be_bytes());
        let data = &data;

        // Encode the message
        let msg = Base64UrlUnpadded::encode_string(data);

        // Create a signature from the raw message bytes
        let sig = self.key.sign(data);
        let sig = Base64UrlUnpadded::encode_string(sig.as_ref());

        // Join the message and signature to create the token
        [msg, sig].join(".")
    }

    pub fn verify_token(&self, token: &str) -> Result<UserId, VerifyError> {
        // Split the token parts
        let (msg_raw, sig_raw) = match token.split_once('.') {
            Some(value) => value,
            None => return Err(VerifyError::Invalid),
        };

        // Decode the 12 byte token message
        let mut msg = [0u8; 12];
        Base64UrlUnpadded::decode(msg_raw, &mut msg).map_err(|_| VerifyError::Invalid)?;

        // Decode 32byte signature (SHA256)
        let mut sig = [0u8; 32];
        Base64UrlUnpadded::decode(sig_raw, &mut sig).map_err(|_| VerifyError::Invalid)?;

        // Verify the signature
        if !self.key.verify(&msg, &sig) {
            return Err(VerifyError::Invalid);
        }

        // Extract ID and expiration from the msg bytes
        let mut id = [0u8; 4];
        id.copy_from_slice(&msg[..4]);
        let id = u32::from_be_bytes(id);

        let mut exp = [0u8; 8];
        exp.copy_from_slice(&msg[4..]);
        let exp = u64::from_be_bytes(exp);

        // Ensure the timestamp is not expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock went backwards")
            .as_secs();

        if exp < now {
            return Err(VerifyError::Expired);
        }

        Ok(id)
    }

    pub fn remove_session(&self, user_id: UserId) {
        let sessions = &mut *self.sessions.lock();
        sessions.remove(&user_id);
    }

    pub fn add_session(&self, user_id: UserId, link: WeakSessionLink) {
        let sessions = &mut *self.sessions.lock();
        sessions.insert(user_id, link);
    }

    /// Currently unused but here for future implementation of invite system for looking
    /// up users to connect players
    #[allow(unused)]
    pub fn lookup_session(&self, user_id: UserId) -> Option<SessionLink> {
        let sessions = &mut *self.sessions.lock();
        let session = sessions.get(&user_id)?;
        let session = match session.upgrade() {
            Some(value) => value,
            // Session has stopped remove it from the map
            None => {
                sessions.remove(&user_id);
                return None;
            }
        };

        Some(session)
    }
}

/// Errors that can occur while verifying a token
#[derive(Debug, Error)]
pub enum VerifyError {
    /// The token is expired
    #[error("Authorization token is expired")]
    Expired,
    /// The token is invalid
    #[error("Invalid authorization token")]
    Invalid,
}

impl HttpError for VerifyError {
    fn status(&self) -> hyper::StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[cfg(test)]
mod test {
    use crate::utils::signing::SigningKey;

    use super::Sessions;

    /// Tests that tokens can be created and verified correctly
    #[test]
    fn test_token() {
        let (key, _) = SigningKey::generate();
        let sessions = Sessions::new(key);

        let player_id = 32;
        let token = sessions.create_token(player_id);
        let claim = sessions.verify_token(&token).unwrap();

        assert_eq!(player_id, claim)
    }
}
