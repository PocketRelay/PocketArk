use uuid::Uuid;

use crate::database_v2::dto::shared_data::{
    CharacterSharedEquipment, CreateSharedDataDto, SharedDataDto, SharedProgression,
};
use crate::database_v2::dto::users::UserId;
use crate::database_v2::{DbExecutor, DbResult};

pub struct SharedDataRepository;

impl SharedDataRepository {
    /// Create shared data for the provided user
    pub async fn create(
        db: impl DbExecutor<'_>,
        create: CreateSharedDataDto,
    ) -> DbResult<SharedDataDto> {
        let shared_equipment = serde_json::to_string(&create.shared_equipment)
            .map_err(|err| sqlx::Error::Encode(err.into()))?;
        let shared_progression = serde_json::to_string(&create.shared_progression)
            .map_err(|err| sqlx::Error::Encode(err.into()))?;
        let shared_stats = serde_json::to_string(&create.shared_stats)
            .map_err(|err| sqlx::Error::Encode(err.into()))?;

        sqlx::query_as(
            r#"
            INSERT INTO "shared_data" (
                "user_id",
                "active_character_id",
                "shared_equipment",
                "shared_progression",
                "shared_stats"
            )
            VALUES (?, NULL, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(create.user_id)
        .bind(shared_equipment)
        .bind(shared_progression)
        .bind(shared_stats)
        .fetch_one(db)
        .await
    }

    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Option<SharedDataDto>> {
        sqlx::query_as(r#"SELECT * FROM "shared_data" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_optional(db)
            .await
    }

    /// Update the active character for the provided user
    pub async fn set_user_active_character(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        active_character: Uuid,
    ) -> DbResult<bool> {
        let result = sqlx::query(
            r#"UPDATE "shared_data" SET "active_character_id" = ? WHERE "user_id" = ?"#,
        )
        .bind(active_character)
        .bind(user_id)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update the active character for the provided user
    pub async fn set_user_shared_progression(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        shared_progression: &[SharedProgression],
    ) -> DbResult<bool> {
        let shared_progression = serde_json::to_string(shared_progression)
            .map_err(|err| sqlx::Error::Encode(err.into()))?;

        let result =
            sqlx::query(r#"UPDATE "shared_data" SET "shared_progression" = ? WHERE "user_id" = ?"#)
                .bind(shared_progression)
                .bind(user_id)
                .execute(db)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update the active character for the provided user
    pub async fn set_user_shared_equipment(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        shared_equipment: CharacterSharedEquipment,
    ) -> DbResult<bool> {
        let shared_equipment = serde_json::to_string(&shared_equipment)
            .map_err(|err| sqlx::Error::Encode(err.into()))?;
        let result =
            sqlx::query(r#"UPDATE "shared_data" SET "shared_equipment" = ? WHERE "user_id" = ?"#)
                .bind(shared_equipment)
                .bind(user_id)
                .execute(db)
                .await?;

        Ok(result.rows_affected() > 0)
    }
}
