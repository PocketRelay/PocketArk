use super::User;
use crate::{
    database::DbResult,
    services::{activity::ChallengeUpdateCounter, challenges::ChallengeProgressUpdate},
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

#[skip_serializing_none]
#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "challenge_progress")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    #[serde(skip)]
    pub user_id: u32,

    pub challenge_id: Uuid,
    pub counters: ChallengeCounters,
    pub state: String,
    pub times_completed: u32,
    pub last_completed: Option<DateTime<Utc>>,
    pub first_completed: Option<DateTime<Utc>>,
    pub last_changed: DateTime<Utc>,
    pub rewarded: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct ChallengeCounters(Vec<ChallengeProgressCounter>);

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeProgressCounter {
    pub name: String,
    pub times_completed: u32,
    pub total_count: u32,
    pub current_count: u32,
    pub target_count: u32,
    pub reset_count: u32,
    pub last_changed: DateTime<Utc>,
}

pub enum ProgressUpdateType {
    Changed,
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
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn find_by_user(db: &DatabaseConnection, user: &User) -> DbResult<Vec<Self>> {
        user.find_related(Entity).all(db).await
    }

    pub async fn handle_update<C>(
        db: &C,
        user: &User,
        update: ChallengeProgressUpdate,
    ) -> DbResult<(Self, ChallengeUpdateCounter, ProgressUpdateType)>
    where
        C: ConnectionTrait + Send,
    {
        let mut update_counter = ChallengeUpdateCounter {
            current_count: update.progress,
            name: update.counter.name.clone(),
        };

        // TODO: Handling for interval field and resetting?

        if let Some(mut progress) = user
            .find_related(Entity)
            .filter(Column::ChallengeId.eq(update.definition.name))
            .one(db)
            .await?
        {
            let now = Utc::now();
            let last_complete = progress.times_completed;
            let mut times_complete = progress.times_completed;
            let counter = progress
                .counters
                .0
                .iter_mut()
                .find(|counter| counter.name.eq(&update.counter.name));

            if let Some(counter) = counter {
                counter.target_count = update.counter.target_count;

                let new_count = counter.current_count.saturating_add(update.progress);
                counter.current_count = new_count.min(counter.target_count);
                counter.total_count = counter.total_count.saturating_add(update.progress);

                update_counter.current_count = counter.current_count;

                if counter.current_count == counter.target_count {
                    counter.times_completed += 1;
                    times_complete += 1;
                }

                counter.last_changed = now;
            }

            let mut model = progress.into_active_model();
            model.times_completed = Set(times_complete);
            model.counters = Set(model.counters.take().expect("Missing counters"));
            model.last_changed = Set(now);

            if times_complete != last_complete {
                model.last_completed = Set(Some(now));
                if times_complete == 1 {
                    model.first_completed = Set(Some(now))
                }
            }

            let model = model.update(db).await?;

            Ok((model, update_counter, ProgressUpdateType::Changed))
        } else {
            let now = Utc::now();

            let counter = ChallengeProgressCounter {
                name: update.counter.name.to_string(),
                times_completed: 0,
                total_count: update.progress,
                current_count: update.progress,
                target_count: update.counter.target_count,
                reset_count: 0,
                last_changed: now,
            };
            let model = ActiveModel {
                id: NotSet,
                user_id: Set(user.id),
                challenge_id: Set(update.definition.name),
                counters: Set(ChallengeCounters(vec![counter])),
                state: Set("IN_PROGRESS".to_string()),
                times_completed: Set(0),
                last_changed: Set(now),
                rewarded: Set(false),
                last_completed: Set(None),
                first_completed: Set(None),
            };
            // todo: apply rewards
            let model = model.insert(db).await?;
            Ok((model, update_counter, ProgressUpdateType::Created))
        }
    }
}
