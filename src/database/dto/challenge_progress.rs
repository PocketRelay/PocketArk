use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use strum::EnumIter;
use uuid::Uuid;

use crate::{
    database::dto::users::UserId, definitions::challenges::ChallengeCounter, utils::ImStr,
};

/// Type alias for a challenge ID
pub type ChallengeId = Uuid;

/// Challenge progress database structure
#[skip_serializing_none]
#[derive(Clone, Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeProgressDto {
    #[serde(skip)]
    pub user_id: UserId,
    pub challenge_id: ChallengeId,
    /// Counter states for the challenge
    #[sqlx(json)]
    pub counters: Vec<ChallengeProgressCounter>,
    /// The current state of the challenge
    pub state: ChallengeState,
    pub times_completed: u32,
    pub last_completed: Option<DateTime<Utc>>,
    pub first_completed: Option<DateTime<Utc>>,
    pub last_changed: DateTime<Utc>,
    pub rewarded: bool,
}

/// Challenge progress database structure
#[derive(Debug, Clone)]
pub struct CreateChallengeProgressDto {
    pub user_id: UserId,
    pub challenge_id: ChallengeId,
    pub counters: Vec<ChallengeProgressCounter>,
    pub state: ChallengeState,
    pub last_changed: DateTime<Utc>,
}

/// Type alias for a [ImStr] representing the name of a [ChallengeProgressCounter]
pub type ChallengeCounterName = ImStr;

/// Action for showing what the progress update was
#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
pub enum CounterUpdateType {
    /// The counter existing and was just updated
    Changed,
    /// The counter didn't exist and was created
    Created,
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeProgressCounter {
    /// The name of this challenge counter
    pub name: ChallengeCounterName,
    /// The number of times completed
    pub times_completed: u32,
    /// The total count towards this counter across all times completed
    ///
    /// ..? Cant this just be: (times_completed * target_count) + current_count
    pub total_count: u32,
    /// The current counter progress
    pub current_count: u32,
    /// The number of times this counter has been reset
    pub reset_count: u32,
    /// The last time this counter was changed
    pub last_changed: DateTime<Utc>,
}

impl ChallengeProgressCounter {
    pub fn new(name: ChallengeCounterName) -> Self {
        Self {
            name,
            times_completed: 0,
            total_count: 0,
            current_count: 0,
            reset_count: 0,
            last_changed: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeProgressCounterWithDefinition {
    #[serde(flatten)]
    pub definition: ChallengeCounter,
    #[serde(flatten)]
    pub counter: ChallengeProgressCounter,
}

/// Enum for the different known challenge states
#[derive(Debug, EnumIter, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[repr(u8)]
pub enum ChallengeState {
    #[serde(rename = "IN_PROGRESS")]
    InProgress = 0,
    #[serde(rename = "COMPLETED")]
    Completed = 1,
    #[serde(rename = "NOT_STARTED")]
    NotStarted = 2,
}
