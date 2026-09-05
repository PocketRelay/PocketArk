use crate::{
    database_v2::{
        DbExecutor, DbResult,
        dto::{
            strike_team_mission::{StrikeTeamMissionProgressDto, UserMissionState},
            strike_teams::{CreateStrikeTeamDto, StrikeTeamDto, StrikeTeamId},
            users::UserId,
        },
        extensions::SqlxBindExt,
    },
    definitions::strike_teams::equipment::StrikeTeamEquipment,
};

pub struct StrikeTeamsRepository;

impl StrikeTeamsRepository {
    pub async fn create(
        db: impl DbExecutor<'_>,
        create: CreateStrikeTeamDto,
    ) -> DbResult<StrikeTeamDto> {
        sqlx::query_as(
            r#"
            INSERT INTO "strike_teams" (
                "user_id", "name", "icon", "level",
                "xp", "positive_traits", "negative_traits"
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#,
        )
        .bind(create.user_id)
        .bind(create.name)
        .bind_json(create.icon)?
        .bind(create.level)
        .bind_json(create.xp)?
        .bind_json(create.positive_traits)?
        .bind_json(create.negative_traits)?
        .fetch_one(db)
        .await
    }

    pub async fn set_equipment(
        db: impl DbExecutor<'_>,
        id: StrikeTeamId,
        equipment: Option<StrikeTeamEquipment>,
    ) -> DbResult<StrikeTeamDto> {
        sqlx::query_as(
            r#"
            UPDATE "strike_teams"
            SET "equipment" = ?
            WHERE "id" = ?
            RETURNING *
        "#,
        )
        .bind_json(equipment)?
        .bind(id)
        .fetch_one(db)
        .await
    }

    pub async fn delete(db: impl DbExecutor<'_>, id: StrikeTeamId) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM "strike_teams"
            WHERE "id" = ?
        "#,
        )
        .bind(id)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_by_id(
        db: impl DbExecutor<'_>,
        id: StrikeTeamId,
    ) -> DbResult<Option<StrikeTeamDto>> {
        sqlx::query_as(
            r#"
            SELECT * FROM "strike_teams"
            WHERE "id" = ?
        "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<StrikeTeamDto>> {
        sqlx::query_as(
            r#"
            SELECT * FROM "strike_teams"
            WHERE "user_id" = ?
        "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await
    }

    pub async fn get_by_user_count(db: impl DbExecutor<'_>, user_id: UserId) -> DbResult<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM "strike_teams"
            WHERE "user_id" = ?
        "#,
        )
        .bind(user_id)
        .fetch_one(db)
        .await
    }

    pub async fn get_active_progress(
        db: impl DbExecutor<'_>,
        team_id: StrikeTeamId,
    ) -> DbResult<Option<StrikeTeamMissionProgressDto>> {
        sqlx::query_as(
            r#"
            SELECT
                "user_mission_state"
                "seen"
                "completed"
            FROM "strike_team_mission_progress"
            WHERE "strike_team_id" = ?
                AND "user_mission_state" IN (?, ?)
            LIMIT 1
        "#,
        )
        .bind(UserMissionState::PendingResolve)
        .bind(UserMissionState::InProgress)
        .bind(team_id)
        .fetch_optional(db)
        .await
    }
}
