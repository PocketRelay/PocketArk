use super::{users::UserId, User};
use crate::{
    database::DbResult,
    definitions::challenges::{ChallengeCounter, ChallengeDefinition, ChallengeName},
    services::game::ChallengeProgressChange,
    utils::ImStr,
};
use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue::Set, FromJsonQueryResult, IntoActiveModel};
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
    pub counters: ChallengeCounters,
    /// The current state of the challenge
    pub state: ChallengeState,
    pub times_completed: u32,
    pub last_completed: Option<DateTimeUtc>,
    pub first_completed: Option<DateTimeUtc>,
    pub last_changed: DateTimeUtc,
    pub rewarded: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct ChallengeCounters(Vec<ChallengeProgressCounter>);

/// Type alias for a [ImStr] representing the name of a [ChallengeProgressCounter]
pub type ChallengeCounterName = ImStr;

/// Action for showing what the progress update was
#[derive(Debug, PartialEq, Eq)]
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
    /// The required count for this challenge to be complete
    pub target_count: u32,
    /// The number of times this counter has been reset
    pub reset_count: u32,
    /// The last time this counter was changed
    pub last_changed: DateTimeUtc,
}

impl ChallengeProgressCounter {
    /// Adds progress to this counter
    pub fn add_progress(&mut self, progress: u32) {
        // Add the progress
        self.total_count = self.total_count.saturating_add(progress);
        self.current_count = self.current_count.saturating_add(progress);
    }

    /// Processes the counter state ensuring that the times completed
    /// and current count are adjusted
    pub fn process(
        &mut self,
        definition: &ChallengeDefinition,
        counter_definition: &ChallengeCounter,
    ) {
        if definition.can_repeat {
            // Handle repeating the task multiple times
            while self.current_count >= counter_definition.target_count {
                // Remove the completed amount
                self.current_count -= counter_definition.target_count;
                // Increase the times completed
                self.times_completed += 1;
            }
        } else if self.current_count > self.target_count {
            self.current_count = self.target_count;
            self.times_completed = 1;
        }
    }
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

    pub async fn update<C>(
        db: &C,
        user: &User,
        change: &ChallengeProgressChange,
    ) -> DbResult<(Self, ChallengeProgressCounter, CounterUpdateType)>
    where
        C: ConnectionTrait + Send,
    {
        // TODO: How are challenges reset?

        let now = Utc::now();

        // Load the challenge
        let mut challenge = Self::get_or_create(db, user, change.definition.name).await?;

        // Take all the counters from the original list
        let mut counters = challenge.counters.0.split_off(0);

        let update_type: CounterUpdateType;

        // Find the counter if it already exists
        let counter = if let Some(existing) = counters
            .iter_mut()
            .find(|counter| counter.name == change.counter.name)
        {
            update_type = CounterUpdateType::Changed;
            existing
        } else {
            // Create a new counter
            update_type = CounterUpdateType::Created;

            counters.push(ChallengeProgressCounter {
                name: change.counter.name.clone(),
                times_completed: 0,
                total_count: 0,
                current_count: 0,
                target_count: 0,
                reset_count: 0,
                last_changed: now,
            });

            counters
                .last_mut()
                .expect("Counter was just inserted but is missing")
        };

        let prev_completion_times = counter.times_completed;

        // Add and update the progression
        counter.add_progress(change.progress);
        counter.process(change.definition, change.counter);
        counter.last_changed = now;

        // Take a copy of the counter for re-use
        let counter = counter.clone();

        // First completion
        let first_completion = prev_completion_times == 0 && counter.times_completed > 0;
        // Challenge counter was completed
        let completed = prev_completion_times != counter.times_completed;

        // Update the stored challenge progress
        let mut model = challenge.into_active_model();
        model.last_changed = Set(now);
        model.times_completed = Set(counter.times_completed);
        model.counters = Set(ChallengeCounters(counters));

        if first_completion {
            model.first_completed = Set(Some(now));
        }

        if completed {
            model.last_completed = Set(Some(now));
            model.state = Set(ChallengeState::Completed);
        }

        let model = model.update(db).await?;
        Ok((model, counter, update_type))
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
