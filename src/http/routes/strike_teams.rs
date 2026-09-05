use crate::{
    database::{
        DbPool,
        dto::{
            currency::{CurrencyDto, CurrencyType},
            strike_team_mission::{StrikeTeamMissionDto, StrikeTeamMissionId, UserMissionState},
            strike_teams::{StrikeTeamDto, StrikeTeamId},
        },
        repositories::{
            strike_team_mission::StrikeTeamMissionRepository, strike_teams::StrikeTeamsRepository,
        },
    },
    definitions::{
        i18n::{I18n, Localized},
        strike_teams::{
            MAX_STRIKE_TEAMS, STRIKE_TEAM_COSTS, StrikeTeams, create_user_strike_team,
            equipment::StrikeTeamEquipment, specialization::StrikeTeamSpecialization,
        },
    },
    http::{
        middleware::user::Auth,
        models::{
            CurrencyError, DynHttpError, HttpResult, RawJson, VecWithCount,
            strike_teams::{
                PurchaseQuery, PurchaseResponse, StrikeTeamError, StrikeTeamMissionSpecific,
                StrikeTeamMissionWithState, StrikeTeamSuccessRate, StrikeTeamWithMission,
                StrikeTeamsList, StrikeTeamsResponse,
            },
        },
    },
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use chrono::{DateTime, Utc};
use log::debug;
use std::{collections::HashMap, ops::DerefMut};
use uuid::Uuid;

use super::store::try_spend_currency;

/// GET /striketeams
pub async fn get(
    Extension(db): Extension<DbPool>,
    Auth(user): Auth,
) -> HttpResult<StrikeTeamsResponse> {
    let strike_teams: Vec<StrikeTeamDto> = StrikeTeamsRepository::get_by_user(&db, user.id).await?;

    // TODO: Load current missions
    let mut teams: Vec<StrikeTeamWithMission> = strike_teams
        .into_iter()
        .map(|team| StrikeTeamWithMission {
            mission: None,
            team,
        })
        .collect();

    teams.localize(I18n::get());

    // Create a map of the next costs
    let next_purchase_costs: HashMap<CurrencyType, u32> = STRIKE_TEAM_COSTS
        .get(teams.len())
        .into_iter()
        .map(|value| (CurrencyType::Mission, *value))
        .collect();

    Ok(Json(StrikeTeamsResponse {
        teams: StrikeTeamsList {
            total_count: teams.len(),
            cap: MAX_STRIKE_TEAMS,
            list: teams,
        },
        min_specialization_level: 16,
        next_purchase_costs,
        inventory_item_limit: 200,
        inventory_item_count: 0,
    }))
}

/// GET /striketeams/successRate
pub async fn get_success_rate(
    Extension(db): Extension<DbPool>,
    Auth(user): Auth,
) -> HttpResult<VecWithCount<StrikeTeamSuccessRate>> {
    let current_time = Utc::now().timestamp();
    let strike_teams = StrikeTeamsRepository::get_by_user(&db, user.id).await?;
    let missions =
        StrikeTeamMissionRepository::get_user_available_missions(&db, user.id, current_time)
            .await?;

    fn compute_success_rate(_strike_team: &StrikeTeamDto, _mission: &StrikeTeamMissionDto) -> f32 {
        // Compute actual success rate
        0.91
    }

    let rates: Vec<StrikeTeamSuccessRate> = strike_teams
        .into_iter()
        .map(|team| {
            let mission_success_rate = missions
                .iter()
                .map(|mission| {
                    let rate = compute_success_rate(&team, &mission.mission);
                    (mission.mission.id, rate)
                })
                .collect();

            StrikeTeamSuccessRate {
                id: team.id,
                name: team.name,
                mission_success_rate,
            }
        })
        .collect();

    Ok(Json(VecWithCount::new(rates)))
}

/// GET /striketeams/missionConfig
pub async fn get_mission_config() -> RawJson {
    static DEFS: &str = include_str!("../../resources/defaults/strikeTeams/missionConfig.json");
    RawJson(DEFS)
}

/// GET /striketeams/specializations
pub async fn get_specializations() -> Json<VecWithCount<StrikeTeamSpecialization>> {
    let strike_teams = StrikeTeams::get();
    let mut specializations = strike_teams.specializations.specializations.clone();
    specializations.localize(I18n::get());
    Json(VecWithCount::new(specializations))
}

/// GET /striketeams/equipment
pub async fn get_equipment() -> Json<VecWithCount<StrikeTeamEquipment>> {
    let strike_teams = StrikeTeams::get();
    let mut equipment = strike_teams.equipment.equipment.clone();
    equipment.localize(I18n::get());
    Json(VecWithCount::new(equipment))
}

/// POST /striketeams/:id/equipment/:name?currency=MissionCurrency
pub async fn purchase_equipment(
    Auth(user): Auth,
    Query(query): Query<PurchaseQuery>,
    Path((id, name)): Path<(StrikeTeamId, String)>,
    Extension(db): Extension<DbPool>,
) -> HttpResult<PurchaseResponse> {
    let strike_teams = StrikeTeams::get();

    // Find the strike team the user wants to equip
    let team = StrikeTeamsRepository::get_by_id(&db, user.id, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    // TODO: Current progress = StrikeTeamsRepository::get_active_progress to properly check this

    // TODO: I don't think this on mission check is correct...?
    let current_progress = StrikeTeamsRepository::get_active_progress(&db, team.id)
        .await
        .map(|value| value.is_some())?;
    if current_progress {
        return Err(StrikeTeamError::TeamOnMission.into());
    }

    let equipment = strike_teams
        .equipment
        .equipment
        .iter()
        .find(|equip| equip.name.eq(&name))
        .ok_or(StrikeTeamError::UnknownEquipmentItem)?;

    let equipment_cost = *equipment
        .cost_by_currency
        .get(&query.currency)
        .ok_or(CurrencyError::InvalidCurrency)?;

    let (team, currency_balance): (StrikeTeamDto, CurrencyDto) = {
        let mut transaction = db.begin().await?;
        // Spend the cost of the strike team equipment
        let currency_balance =
            try_spend_currency(&mut transaction, &user, query.currency, equipment_cost).await?;

        // Assign the equipment to the team
        let team = StrikeTeamsRepository::set_equipment(
            transaction.deref_mut(),
            team.id,
            Some(equipment.clone()),
        )
        .await?;

        transaction.commit().await?;

        (team, currency_balance)
    };

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost: Some(0),
    }))
}

/// POST /striketeams/:id/mission/resolve
pub async fn resolve_mission(Path(id): Path<Uuid>) -> RawJson {
    debug!("Strike team mission resolve: {}", id);

    // TODO: Handle resolving a mission in pending resolve state
    // updating to completed state and granting rewards

    static DEFS: &str =
        include_str!("../../resources/defaults/strikeTeams/placeholderResolve.json");
    RawJson(DEFS)
}

/// POST /striketeams/:id/mission/:id
///
/// Obtain the details about a specific strike team mission
pub async fn get_mission(
    Auth(user): Auth,
    Path((id, mission_id)): Path<(StrikeTeamId, StrikeTeamMissionId)>,
    Extension(db): Extension<DbPool>,
) -> HttpResult<StrikeTeamMissionSpecific> {
    debug!("Strike team get mission : {} {}", id, mission_id);

    let mission = StrikeTeamMissionRepository::get_by_id(&db, mission_id)
        .await?
        .ok_or(StrikeTeamError::UnknownMission)?;
    let strike_team = StrikeTeamsRepository::get_by_id(&db, user.id, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    let progress = StrikeTeamsRepository::get_active_progress(&db, strike_team.id).await?;

    let mut live_mission = match progress {
        Some(value) => StrikeTeamMissionWithState {
            mission,
            user_mission_state: value.user_mission_state,
            seen: value.seen,
            completed: value.completed,
        },
        None => StrikeTeamMissionWithState {
            mission,
            user_mission_state: UserMissionState::Available,
            seen: false,
            completed: false,
        },
    };

    live_mission.localize(I18n::get());

    let finish_time: DateTime<Utc> = Utc::now(); /* TODO: Proper finish time */

    Ok(Json(StrikeTeamMissionSpecific {
        name: mission_id,
        live_mission,
        finish_time,
    }))
}

/// POST /striketeams/:id/retire
///
/// Retires (Removes) a strike team from the players
/// strike teams
pub async fn retire(
    Auth(user): Auth,
    Path(id): Path<StrikeTeamId>,
    Extension(db): Extension<DbPool>,
) -> Result<(), DynHttpError> {
    debug!("Strike team retire: {}", id);
    let _team = StrikeTeamsRepository::get_by_id(&db, user.id, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    StrikeTeamsRepository::delete(&db, id).await?;
    Ok(())
}

/// POST /striketeams/purchase?currency=MissionCurrency
pub async fn purchase(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<PurchaseResponse> {
    // Get the number of teams they already have
    let strike_teams = StrikeTeamsRepository::get_by_user_count(&db, user.id).await? as usize;

    // Get the cost of a new team
    let strike_team_cost = *STRIKE_TEAM_COSTS
        .get(strike_teams)
        .ok_or(StrikeTeamError::MaxTeams)?;

    let (mut team, currency_balance): (StrikeTeamDto, CurrencyDto) = {
        let mut transaction = db.begin().await?;
        // Spend the cost of the strike team
        let currency_balance = try_spend_currency(
            &mut transaction,
            &user,
            CurrencyType::Mission,
            strike_team_cost,
        )
        .await?;

        // Create the strike team
        let team = create_user_strike_team(&mut transaction, &user).await?;

        transaction.commit().await?;

        (team, currency_balance)
    };

    team.localize(I18n::get());

    // Get the cost of the next team
    let next_purchase_cost = STRIKE_TEAM_COSTS.get(strike_teams + 1).copied();

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost,
    }))
}
