use crate::blaze::models::{game_manager::GameManagerError, user_sessions::UserSessionsError};
use pocket_ark_blaze_networking::error::{BlazeError, CodedBlazeError};
use pocket_ark_database::DbErr;
use tracing::error;

pub use pocket_ark_blaze_networking::error::ServerResult;

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

pub struct BlazeDbErr(DatabaseError);

impl From<DbErr> for BlazeDbErr {
    fn from(value: DbErr) -> Self {
        error!("Database error: {}", value);
        BlazeDbErr(DatabaseError::System.into())
    }
}

impl CodedBlazeError for BlazeDbErr {
    fn error_code(self) -> u32 {
        self.0 as u32
    }
}

impl From<DatabaseError> for BlazeError {
    fn from(value: DatabaseError) -> Self {
        BlazeError(value as u32)
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
