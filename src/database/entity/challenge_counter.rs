use std::future::Future;

use super::{challenge_progress::ChallengeId, users::UserId, ChallengeProgress, User};
use crate::{
    database::DbResult,
    services::{
        activity::{ChallengeStatusChange, ChallengeUpdateCounter},
        challenges::ChallengeName,
        game::ChallengeProgressChange,
    },
};
use chrono::{DateTime, Utc};
use sea_orm::{
    entity::prelude::*,
    ActiveValue::{NotSet, Set},
    FromJsonQueryResult, IntoActiveModel,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

/// Type alias for a [String] representing the name of a [ChallengeProgressCounter]
pub type ChallengeCounterName = String;

/// Challenge counter database structure
#[skip_serializing_none]
#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "challenge_counter")]
pub struct Model {
    /// The user this challenge counter belongs to
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub user_id: UserId,
    /// The challenge this counter belongs to
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub challenge_id: ChallengeId,
    /// The name of this challenge counter
    #[sea_orm(primary_key)]
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
    pub last_changed: DateTime<Utc>,
}

/// Action for showing what the progress update was
#[derive(Debug, PartialEq, Eq)]
pub enum CounterUpdateType {
    /// The counter existing and was just updated
    Changed,
    /// The counter didn't exist and was created
    Created,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::challenge_progress::Entity",
        from = "(Column::UserId, Column::ChallengeId)",
        to = "(super::challenge_progress::Column::UserId, super::challenge_progress::Column::ChallengeId)"
    )]
    Challenge,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl Related<super::challenge_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Challenge.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Obtains all the counter for the specific `user` and `challenge`
    pub fn all<'db, C>(
        db: &'db C,
        user: &User,
        challenge: ChallengeId,
    ) -> impl Future<Output = DbResult<Vec<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .filter(
                Column::UserId
                    .eq(user.id)
                    .and(Column::ChallengeId.eq(challenge)),
            )
            .all(db)
    }

    pub fn get<'db, C>(
        db: &'db C,
        user: &User,
        challenge: ChallengeId,
        name: &ChallengeCounterName,
    ) -> impl Future<Output = DbResult<Option<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::find()
            .filter(
                Column::UserId
                    .eq(user.id)
                    .and(Column::ChallengeId.eq(challenge))
                    .and(Column::Name.eq(name)),
            )
            .one(db)
    }

    /// Increases the counter
    ///
    /// Returns the counter, the type of update, whether the counter was
    /// completed for the first time, and whether the counter increased its
    /// number of completions
    pub async fn increase<C>(
        db: &C,
        user: &User,
        change: &ChallengeProgressChange,
    ) -> DbResult<(Self, CounterUpdateType, u32, u32)>
    where
        C: ConnectionTrait + Send,
    {
        let now = Utc::now();

        let existing = Self::get(db, user, change.challenge, &change.counter_name).await?;

        if let Some(existing) = existing {
            // The original number of completions
            let mut original_times = existing.times_completed;

            let mut progress = change.progress;

            let mut times_completed = existing.times_completed;
            let mut current_count = existing.current_count.saturating_add(progress);
            let mut total_count = existing.total_count.saturating_add(progress);

            // Handle repeated
            if change.can_repeat {
                // Handle completions
                while current_count > change.target_count {
                    // Remove the target amount
                    current_count -= change.target_count;
                    // Increase the times completed
                    times_completed += 1;
                }
            } else if current_count > change.target_count {
                current_count = change.target_count;
                times_completed = 1;
            }

            // Save the changes to the database
            let mut model = existing.into_active_model();
            model.times_completed = Set(times_completed);
            model.current_count = Set(current_count);
            model.total_count = Set(total_count);
            // Override existing target count (DB is not source of truth)
            model.target_count = Set(change.target_count);
            model.last_changed = Set(now);
            let model = model.update(db).await?;

            Ok((
                model,
                CounterUpdateType::Changed,
                original_times,
                times_completed,
            ))
        } else {
            let mut times_completed = 0;
            let mut current_count = change.progress;

            // Handle repeated
            if change.can_repeat {
                // Handle completions
                while current_count > change.target_count {
                    // Remove the target amount
                    current_count -= change.target_count;
                    // Increase the times completed
                    times_completed += 1;
                }
            } else if current_count > change.target_count {
                current_count = change.target_count;
                times_completed = 1;
            }

            // Create new model
            Entity::insert(ActiveModel {
                user_id: Set(user.id),
                challenge_id: Set(change.challenge),
                name: Set(change.counter_name.clone()),
                total_count: Set(change.progress),
                current_count: Set(current_count),
                target_count: Set(change.target_count),
                last_changed: Set(now),
                times_completed: Set(times_completed),
                ..Default::default()
            })
            // Returning doesn't work with composite key
            .exec_without_returning(db)
            .await?;

            // Counter must be loaded manually
            let counter = Self::get(db, user, change.challenge, &change.counter_name)
                .await?
                .ok_or(DbErr::RecordNotInserted)?;

            Ok((counter, CounterUpdateType::Created, 0, times_completed))
        }
    }
}
