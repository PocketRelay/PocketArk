use std::future::Future;

use super::{
    challenge_counter::{ChallengeCounterName, CounterUpdateType},
    users::UserId,
    ChallengeCounter, ChallengeProgress, User,
};
use crate::{
    database::DbResult,
    services::{
        activity::{ChallengeStatusChange, ChallengeUpdateCounter},
        challenges::{ChallengeName, ChallengeProgressUpdate},
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

/// Type alias for a challenge ID
pub type ChallengeId = Uuid;

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
    /// The current state of the challenge
    pub state: ChallengeState,
    pub times_completed: u32,
    pub last_completed: Option<DateTime<Utc>>,
    pub first_completed: Option<DateTime<Utc>>,
    pub last_changed: DateTime<Utc>,
    pub rewarded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChallengeProgressWithCounters {
    /// The challenge progress
    #[serde(flatten)]
    pub progress: Model,
    /// The counters associated with this challenge progress
    pub counters: Vec<ChallengeCounter>,
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

    #[sea_orm(has_many = "super::challenge_counter::Entity")]
    Counter,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl Related<super::challenge_counter::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Counter.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn find_by_user(db: &DatabaseConnection, user: &User) -> DbResult<Vec<Self>> {
        user.find_related(Entity).all(db).await
    }

    pub async fn all_with_counters<C>(
        db: &C,
        user: &User,
    ) -> DbResult<Vec<ChallengeProgressWithCounters>>
    where
        C: ConnectionTrait + Send,
    {
        let values = user
            .find_related(Entity)
            .find_with_related(super::challenge_counter::Entity)
            .all(db)
            .await?
            .into_iter()
            .map(|(progress, counters)| ChallengeProgressWithCounters { progress, counters })
            .collect();

        Ok(values)
    }

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

    pub async fn update<C>(
        db: &C,
        user: &User,
        challenge: ChallengeId,
        counter_name: ChallengeCounterName,
        progress: u32,
        counter_target: u32,
    ) -> DbResult<(Self, ChallengeCounter, CounterUpdateType)>
    where
        C: ConnectionTrait + Send,
    {
        // TODO: How are challenges reset?

        let now = Utc::now();

        // Update the counter value
        let (counter, update_type, original_times, times_completed) =
            ChallengeCounter::increase(db, user, challenge, counter_name, progress, counter_target)
                .await?;

        // First completion
        let first_completion = original_times == 0 && times_completed > 0;
        // Challenge counter was completed
        let completed = original_times != times_completed;

        let existing = Self::get(db, user, challenge).await?;
        let model = if let Some(existing) = existing {
            let mut model = existing.into_active_model();
            model.times_completed = Set(times_completed);
            model.last_changed = Set(now);

            // Update completion times
            if first_completion {
                model.first_completed = Set(Some(now));
            }

            if completed {
                model.last_completed = Set(Some(now));
                model.state = Set(ChallengeState::Completed);
            }

            model.update(db).await?
        } else {
            // Create new model
            Entity::insert(ActiveModel {
                user_id: Set(user.id),
                challenge_id: Set(challenge),
                state: Set(if completed {
                    ChallengeState::Completed
                } else {
                    ChallengeState::InProgress
                }),
                times_completed: Set(times_completed),
                last_changed: Set(now),
                last_completed: Set(if completed { Some(now) } else { None }),
                first_completed: Set(if first_completion { Some(now) } else { None }),
                rewarded: Set(false),
                ..Default::default()
            })
            // Returning doesn't work with composite key
            .exec_without_returning(db)
            .await?;

            // Progress must be loaded manually
            Self::get(db, user, challenge)
                .await?
                .ok_or(DbErr::RecordNotInserted)?
        };
        Ok((model, counter, update_type))
    }
}
