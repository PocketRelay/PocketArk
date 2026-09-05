use crate::{
    database::{
        DbPool, dto::strike_team_mission::StrikeTeamMissionWithProgressDto,
        repositories::strike_team_mission::StrikeTeamMissionRepository,
    },
    definitions::{
        i18n::{I18n, Localized},
        mission::tutorial::get_tutorial_mission,
    },
    http::{
        middleware::{JsonDump, user::Auth},
        models::{
            VecWithCount,
            errors::{DynHttpError, HttpResult},
            mission::*,
            strike_teams::StrikeTeamMissionWithState,
        },
    },
    services::game::{data::process_mission_data, store::Games},
};
use axum::{Extension, Json, extract::Path};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use serde_json::Value;
use std::sync::Arc;

/// GET /mission/current
///
/// Obtains a list of currently available missions
pub async fn current_missions(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<VecWithCount<StrikeTeamMissionWithState>> {
    let current_time = Utc::now().timestamp();
    let missions =
        StrikeTeamMissionRepository::get_user_visible_missions(&db, user.id, current_time).await?;

    let mut missions: Vec<StrikeTeamMissionWithState> = missions
        .into_iter()
        .map(
            |StrikeTeamMissionWithProgressDto { mission, progress }| StrikeTeamMissionWithState {
                mission,
                user_mission_state: progress.user_mission_state,
                seen: progress.seen,
                completed: progress.completed,
            },
        )
        .collect();

    missions.localize(I18n::get());

    let tutorial_mission: StrikeTeamMissionWithState = get_tutorial_mission();
    missions.push(tutorial_mission);

    Ok(Json(VecWithCount::new(missions)))
}

/// GET /user/mission/:id
///
/// Obtains the details about a specific mission
///
/// Called at end of game to obtain information about the
/// game and rewards etc
pub async fn get_mission(
    Path(mission_id): Path<u32>,
    Extension(games): Extension<Arc<Games>>,
) -> HttpResult<MissionDetails> {
    debug!("Requested mission details: {}", mission_id);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    let game = game.read();

    let mission_data = game
        .get_processed_data()
        .ok_or(MissionError::MissingMissionData)?;

    Ok(Json(mission_data))
}

/// POST /user/mission/:id/start
///
/// Starts a mission
pub async fn start_mission(
    Path(mission_id): Path<u32>,
    Extension(games): Extension<Arc<Games>>,
    JsonDump(req): JsonDump<StartMissionRequest>,
) -> HttpResult<StartMissionResponse> {
    debug!("Mission started: {} {:?}", mission_id, req);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    {
        game.write().set_modifiers(req.modifiers);
    }

    let res = StartMissionResponse {
        match_id: mission_id.to_string(),
    };
    Ok(Json(res))
}

/// POST /user/mission/:id/finish
///
/// Submits the details of a mission that has been finished
pub async fn finish_mission(
    Path(mission_id): Path<u32>,
    Extension(db): Extension<DbPool>,
    Extension(games): Extension<Arc<Games>>,
    JsonDump(req): JsonDump<CompleteMissionData>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Mission finished: {} {:#?}", mission_id, req);

    let game = games
        .get_by_id(mission_id)
        .ok_or(MissionError::UnknownGame)?;

    let complete_data = req;
    {
        let mut transaction = db.begin().await?;
        let mission_data = process_mission_data(&mut transaction, complete_data).await;
        debug!(
            "Processed mission data OUTPUT: {}",
            serde_json::to_string(&mission_data).unwrap()
        );
        transaction.commit().await?;

        let game = &mut *game.write();
        game.set_processed(mission_data);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /mission/seen
pub async fn update_seen(JsonDump(req): JsonDump<Value>) -> StatusCode {
    debug!("Update mission seen: {:?}", req);
    StatusCode::NO_CONTENT
}
