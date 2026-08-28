use super::{SeaJson, User, users::UserId};
use crate::{
    database::DbResult,
    definitions::challenges::{ChallengeCounter, ChallengeName},
    services::challenges::AppliedChallengeProgressUpdate,
    utils::ImStr,
};
use chrono::Utc;
use sea_orm::{ActiveValue::Set, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::future::Future;
use uuid::Uuid;

/// Type alias for a challenge ID
pub type ChallengeId = Uuid;

/// Challenge progress database structure
#[skip_serializing_none]
#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "challenge_progress")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub user_id: UserId,
    #[sea_orm(primary_key)]
    pub challenge_id: ChallengeId,
    /// Counter states for the challenge
    pub counters: SeaJson<Vec<ChallengeProgressCounter>>,
    /// The current state of the challenge
    pub state: ChallengeState,
    pub times_completed: u32,
    pub last_completed: Option<DateTimeUtc>,
    pub first_completed: Option<DateTimeUtc>,
    pub last_changed: DateTimeUtc,
    pub rewarded: bool,
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
    pub last_changed: DateTimeUtc,
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
#[derive(
    Debug, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[repr(u8)]
pub enum ChallengeState {
    #[serde(rename = "IN_PROGRESS")]
    InProgress = 0,
    #[serde(rename = "COMPLETED")]
    Completed = 1,
    #[serde(rename = "NOT_STARTED")]
    NotStarted = 2,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Model {
    /// Obtains all the challenge progress (and associated counters) that
    /// belong to the provided `user`
    pub fn all<'db, C>(db: &'db C, user: &User) -> impl Future<Output = DbResult<Vec<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db)
    }

    /// Finds a specific [ChallengeProgress] by ID
    pub fn get<'db, C>(
        db: &'db C,
        user: &User,
        challenge: ChallengeId,
    ) -> impl Future<Output = DbResult<Option<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .filter(
                Column::UserId
                    .eq(user.id)
                    .and(Column::ChallengeId.eq(challenge)),
            )
            .one(db)
    }

    pub async fn get_or_create<C>(db: &C, user: &User, challenge: ChallengeName) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        // Find an existing model
        if let Some(existing) = Self::get(db, user, challenge).await? {
            return Ok(existing);
        }

        let now = Utc::now();
        // Create new model
        Entity::insert(ActiveModel {
            user_id: Set(user.id),
            challenge_id: Set(challenge),
            state: Set(ChallengeState::InProgress),
            counters: Set(Default::default()),
            times_completed: Set(0),
            last_changed: Set(now),
            last_completed: Set(None),
            first_completed: Set(None),
            rewarded: Set(false),
        })
        // Returning doesn't work with composite key
        .exec_without_returning(db)
        .await?;

        // Progress must be loaded manually
        Self::get(db, user, challenge)
            .await?
            .ok_or(DbErr::RecordNotInserted)
    }

    pub async fn update<C>(self, db: &C, change: AppliedChallengeProgressUpdate) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        // Update the stored challenge progress
        let mut model = self.into_active_model();
        model.last_changed = Set(change.last_changed);
        model.times_completed = Set(change.times_completed);
        model.counters = Set(SeaJson(change.counters));
        model.first_completed = Set(change.first_completed);
        model.last_completed = Set(change.last_completed);
        model.state = Set(change.state);
        let model = model.update(db).await?;
        Ok(model)
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
