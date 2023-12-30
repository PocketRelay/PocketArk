use crate::{
    database::entity::{
        currency::CurrencyType, strike_team_mission::StrikeTeamMissionId,
        strike_team_mission_progress::UserMissionState, strike_teams::StrikeTeamId, Currency,
        StrikeTeam, StrikeTeamMission, StrikeTeamMissionProgress,
    },
    definitions::{
        strike_teams::StrikeTeamWithMission,
        strike_teams::{
            create_user_strike_team, StrikeTeamEquipment, StrikeTeamSpecialization, StrikeTeams,
            MAX_STRIKE_TEAMS, STRIKE_TEAM_COSTS,
        },
    },
    http::{
        middleware::user::Auth,
        models::{
            strike_teams::{
                PurchaseQuery, PurchaseResponse, StrikeTeamError, StrikeTeamMissionSpecific,
                StrikeTeamMissionWithState, StrikeTeamSuccessRate, StrikeTeamsList,
                StrikeTeamsResponse,
            },
            CurrencyError, DynHttpError, HttpResult, ListWithCount, RawJson, VecWithCount,
        },
    },
};
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use chrono::Utc;
use log::debug;
use sea_orm::{prelude::DateTimeUtc, DatabaseConnection, TransactionTrait};
use std::collections::HashMap;
use uuid::Uuid;

use super::store::try_spend_currency;

/// GET /striketeams
pub async fn get(
    Extension(db): Extension<DatabaseConnection>,
    Auth(user): Auth,
) -> HttpResult<StrikeTeamsResponse> {
    let strike_teams: Vec<StrikeTeam> = StrikeTeam::get_by_user(&db, &user).await?;

    // TODO: Load current missions
    let teams: Vec<StrikeTeamWithMission> = strike_teams
        .into_iter()
        .map(|team| StrikeTeamWithMission {
            mission: None,
            team,
        })
        .collect();

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
    Extension(db): Extension<DatabaseConnection>,
    Auth(user): Auth,
) -> HttpResult<VecWithCount<StrikeTeamSuccessRate>> {
    let current_time = Utc::now().timestamp();
    let strike_teams = StrikeTeam::get_by_user(&db, &user).await?;
    let missions = StrikeTeamMission::available_missions(&db, &user, current_time).await?;

    fn compute_success_rate(_strike_team: &StrikeTeam, _mission: &StrikeTeamMission) -> f32 {
        // Compute actual success rate
        1.0
    }

    let rates: Vec<StrikeTeamSuccessRate> = strike_teams
        .into_iter()
        .map(|team| {
            let mission_success_rate = missions
                .iter()
                .map(|(mission, _)| {
                    let rate = compute_success_rate(&team, mission);
                    (mission.id, rate)
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
pub async fn get_specializations() -> Json<ListWithCount<StrikeTeamSpecialization>> {
    let strike_teams = StrikeTeams::get();

    Json(ListWithCount::new(&strike_teams.specializations))
}

/// GET /striketeams/equipment
pub async fn get_equipment() -> Json<ListWithCount<StrikeTeamEquipment>> {
    let strike_teams = StrikeTeams::get();
    Json(ListWithCount::new(&strike_teams.equipment))
}

/// POST /striketeams/:id/equipment/:name?currency=MissionCurrency
pub async fn purchase_equipment(
    Auth(user): Auth,
    Query(query): Query<PurchaseQuery>,
    Path((id, name)): Path<(StrikeTeamId, String)>,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<PurchaseResponse> {
    let strike_teams = StrikeTeams::get();

    // Find the strike team the user wants to equip
    let team = StrikeTeam::get_by_id(&db, &user, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    if team.is_on_mission(&db).await? {
        return Err(StrikeTeamError::TeamOnMission.into());
    }

    let equipment = strike_teams
        .equipment
        .iter()
        .find(|equip| equip.name.eq(&name))
        .ok_or(StrikeTeamError::UnknownEquipmentItem)?;

    let equipment_cost = *equipment
        .cost_by_currency
        .get(&query.currency)
        .ok_or(CurrencyError::InvalidCurrency)?;

    let (team, currency_balance): (StrikeTeam, Currency) = db
        .transaction(|db| {
            Box::pin(async move {
                // Spend the cost of the strike team equipment
                let currency_balance =
                    try_spend_currency(db, &user, query.currency, equipment_cost).await?;

                // Assign the equipment to the team
                let team = team.set_equipment(db, Some(equipment.clone())).await?;

                Ok::<_, DynHttpError>((team, currency_balance))
            })
        })
        .await?;

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
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<StrikeTeamMissionSpecific> {
    debug!("Strike team get mission : {} {}", id, mission_id);

    let mission = StrikeTeamMission::by_id(&db, mission_id)
        .await?
        .ok_or(StrikeTeamError::UnknownMission)?;
    let strike_team = StrikeTeam::get_by_id(&db, &user, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;
    let progress = StrikeTeamMissionProgress::get_by_team(&db, &strike_team).await?;

    let live_mission = match progress {
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

    let finish_time: DateTimeUtc = Utc::now(); /* TODO: Proper finish time */

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
    Extension(db): Extension<DatabaseConnection>,
) -> Result<(), DynHttpError> {
    debug!("Strike team retire: {}", id);
    let team = StrikeTeam::get_by_id(&db, &user, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    team.delete(&db).await?;

    Ok(())
}

/// POST /striketeams/purchase?currency=MissionCurrency
pub async fn purchase(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<PurchaseResponse> {
    // Get the number of teams they already have
    let strike_teams = StrikeTeam::get_user_count(&db, &user).await? as usize;

    // Get the cost of a new team
    let strike_team_cost = *STRIKE_TEAM_COSTS
        .get(strike_teams)
        .ok_or(StrikeTeamError::MaxTeams)?;

    let (team, currency_balance): (StrikeTeam, Currency) = db
        .transaction(|db| {
            Box::pin(async move {
                // Spend the cost of the strike team
                let currency_balance =
                    try_spend_currency(db, &user, CurrencyType::Mission, strike_team_cost).await?;

                // Create the strike team
                let team = create_user_strike_team(db, &user).await?;

                Ok::<_, DynHttpError>((team, currency_balance))
            })
        })
        .await?;

    // Get the cost of the next team
    let next_purchase_cost = STRIKE_TEAM_COSTS.get(strike_teams + 1).copied();

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost,
    }))
}
