use crate::database_v2::dto::challenge_progress::{ChallengeId, ChallengeProgressDto};
use crate::database_v2::dto::users::UserId;
use crate::database_v2::extensions::SqlxBindExt;
use crate::database_v2::{DbExecutor, DbResult};
use crate::services::challenges::AppliedChallengeProgressUpdate;

pub struct ChallengeProgressRepository;

impl ChallengeProgressRepository {
    pub async fn create(
        db: impl DbExecutor<'_>,
        create: ChallengeProgressDto,
    ) -> DbResult<ChallengeProgressDto> {
        sqlx::query_as(
            r#"
            INSERT INTO "challenge_progress (
                "user_id", "challenge_id", "state",
                "counters", "last_changed"
            )
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
        "#,
        )
        .fetch_one(db)
        .await
    }

    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<ChallengeProgressDto>> {
        sqlx::query_as(r#"SELECT * FROM "challenge_progress" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_all(db)
            .await
    }

    pub async fn get_by_user_by_id(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        challenge_id: ChallengeId,
    ) -> DbResult<Vec<ChallengeProgressDto>> {
        sqlx::query_as(
            r#"SELECT * FROM "challenge_progress" WHERE "user_id" = ? AND "challenge_id" = ?"#,
        )
        .bind(user_id)
        .bind(challenge_id)
        .fetch_all(db)
        .await
    }

    pub async fn update(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        challenge_id: ChallengeId,
        update: AppliedChallengeProgressUpdate,
    ) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE "challenge_progress"
            SET
                "last_changed" = ?,
                "times_completed" = ?,
                "counters" = ?,
                "first_completed" = ?,
                "last_completed" = ?,
                "state" = ?
            WHERE "user_id" = ? AND "challenge_id" = ?
        "#,
        )
        .bind(update.last_changed)
        .bind(update.times_completed)
        .bind_json(update.counters)?
        .bind(update.first_completed)
        .bind(update.last_completed)
        .bind_json(update.state)?
        .bind(user_id)
        .bind(challenge_id)
        .execute(db)
        .await?;

        Ok(())
    }
}
