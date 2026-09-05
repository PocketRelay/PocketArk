use itertools::Itertools;
use sqlx::AssertSqlSafe;

use crate::database_v2::dto::strike_team_mission::{
    CreateStrikeTeamMissionDto, StrikeTeamMissionDto, StrikeTeamMissionId,
    StrikeTeamMissionProgressDto, StrikeTeamMissionWithProgressDto, UserMissionState,
};
use crate::database_v2::dto::users::UserId;
use crate::database_v2::extensions::SqlxBindExt;
use crate::database_v2::{DbExecutor, DbResult};

pub struct StrikeTeamMissionRepository;

impl StrikeTeamMissionRepository {
    pub async fn create_many(
        db: impl DbExecutor<'_>,
        missions: Vec<CreateStrikeTeamMissionDto>,
    ) -> DbResult<Vec<StrikeTeamMissionDto>> {
        let placeholders = missions
            .iter()
            .map(|_| "(?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .join(",");

        let query = format!(
            r#"
            INSERT INTO "strike_team_missions" (
                "name", "descriptor", "mission_type", "tags",
                "accessibility", "static_modifiers", "dynamic_modifiers",
                "rewards", "custom_attributes", "waves", "start_seconds",
                "end_seconds", "sp_length_seconds"
            )
            VALUES {placeholders}
            RETURNING *
        "#
        );
        let query = AssertSqlSafe(query);

        let mut query = sqlx::query_as(query);
        for mission in missions {
            query = query
                .bind(mission.name)
                .bind_json(mission.descriptor)?
                .bind_json(mission.mission_type)?
                .bind_json(mission.tags)?
                .bind(mission.accessibility)
                .bind_json(mission.static_modifiers)?
                .bind_json(mission.dynamic_modifiers)?
                .bind_json(mission.rewards)?
                .bind_json(mission.custom_attributes)?
                .bind_json(mission.waves)?
                .bind(mission.start_seconds)
                .bind(mission.end_seconds)
                .bind(mission.sp_length_seconds)
        }

        query.fetch_all(db).await
    }

    pub async fn get_by_id(
        db: impl DbExecutor<'_>,
        mission_id: StrikeTeamMissionId,
    ) -> DbResult<Option<StrikeTeamMissionDto>> {
        sqlx::query_as(r#"SELECT * FROM "strike_team_missions" WHERE "id" = ?"#)
            .bind(mission_id)
            .fetch_optional(db)
            .await
    }

    /// Gets all missions that are still available
    ///
    /// TODO: Confirm logic of this query
    pub async fn get_user_visible_missions(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        current_time: i64,
    ) -> DbResult<Vec<StrikeTeamMissionWithProgressDto>> {
        sqlx::query_as(
            r#"
            SELECT
                "m".*,
                COALESCE("p"."user_mission_state", 0) AS "user_mission_state",
                COALESCE("p"."seen", FALSE) AS "seen",
                COALESCE("p"."completed", FALSE) AS "completed"
            FROM "strike_team_missions" "m"
            LEFT JOIN "strike_team_mission_progress" "p"
                ON "p"."mission_id" = "m"."id"
                AND "p"."user_id" = ?
            WHERE
                "m"."end_seconds" > ?
                OR "p"."user_mission_state" IN (?, ?)
        "#,
        )
        .bind(user_id)
        .bind(current_time)
        .bind(UserMissionState::PendingResolve)
        .bind(UserMissionState::InProgress)
        .fetch_all(db)
        .await
    }

    /// Gets all missions that are still available
    ///
    /// TODO: Confirm the logic of this query, should it return in progress missions?
    pub async fn get_user_available_missions(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        current_time: i64,
    ) -> DbResult<Vec<StrikeTeamMissionWithProgressDto>> {
        sqlx::query_as(
            r#"
            SELECT
                "m".*,
                COALESCE("p"."user_mission_state", ?) AS "user_mission_state",
                COALESCE("p"."seen", FALSE) AS "seen",
                COALESCE("p"."completed", FALSE) AS "completed"
            FROM "strike_team_missions" "m"
            LEFT JOIN "strike_team_mission_progress" "p"
                ON "p"."mission_id" = "m"."id"
                AND "p"."user_id" = ?
            WHERE
                "m"."end_seconds" > ?
                AND ("p"."user_mission_state" IS NULL OR "p"."user_mission_state" = ?)
        "#,
        )
        .bind(UserMissionState::Available)
        .bind(user_id)
        .bind(current_time)
        .bind(UserMissionState::Available)
        .fetch_all(db)
        .await
    }

    /// Gets the start seconds timestamp of the mission with the newest
    /// start seconds timestamp
    pub async fn get_newest_mission_timestamp(db: impl DbExecutor<'_>) -> DbResult<Option<i64>> {
        sqlx::query_scalar(
            r#"
            SELECT "start_seconds"
            FROM "strike_team_missions"
            ORDER BY "start_seconds" DESC
            LIMIT 1
        "#,
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_user_mission_progress(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        mission_id: StrikeTeamMissionId,
    ) -> DbResult<Vec<StrikeTeamMissionProgressDto>> {
        sqlx::query_as(
            r#"
            SELECT
                COALESCE("p"."user_mission_state", ?) AS "user_mission_state",
                COALESCE("p"."seen", FALSE) AS "seen",
                COALESCE("p"."completed", FALSE) AS "completed"
            FROM "strike_team_missions" "m"
            LEFT JOIN "strike_team_mission_progress" "p"
                ON "p"."mission_id" = "m"."id"
                AND "p"."user_id" = ?
            WHERE "m"."id" = ?
        "#,
        )
        .bind(UserMissionState::Available)
        .bind(user_id)
        .bind(mission_id)
        .fetch_all(db)
        .await
    }
}
