use crate::{
    database::entity::{currency::CurrencyType, strike_teams::StrikeTeamId, Currency, StrikeTeam},
    definitions::{
        strike_teams::create_user_strike_team,
        striketeams::{
            StrikeTeamDefinitions, StrikeTeamEquipment, StrikeTeamSpecialization,
            StrikeTeamWithMission,
        },
    },
    http::{
        middleware::user::Auth,
        models::{
            strike_teams::{
                PurchaseQuery, PurchaseResponse, StrikeTeamError, StrikeTeamsList,
                StrikeTeamsResponse,
            },
            CurrencyError, DynHttpError, HttpResult, ListWithCount, RawJson,
        },
    },
};
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use log::debug;
use sea_orm::{DatabaseConnection, TransactionTrait};
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
    let next_purchase_costs: HashMap<CurrencyType, u32> = StrikeTeamDefinitions::STRIKE_TEAM_COSTS
        .get(teams.len())
        .into_iter()
        .map(|value| (CurrencyType::Mission, *value))
        .collect();

    Ok(Json(StrikeTeamsResponse {
        teams: StrikeTeamsList {
            total_count: teams.len(),
            cap: StrikeTeamDefinitions::MAX_STRIKE_TEAMS,
            list: teams,
        },
        min_specialization_level: 16,
        next_purchase_costs,
        inventory_item_limit: 200,
        inventory_item_count: 0,
    }))
}

/// GET /striketeams/successRate
pub async fn get_success_rate() -> RawJson {
    // TODO: Calculate the success rate for each strike team against each mission

    static DEFS: &str = include_str!("../../resources/data/strikeTeams/successRate.json");
    RawJson(DEFS)
}

/// GET /striketeams/missionConfig
pub async fn get_mission_config() -> RawJson {
    static DEFS: &str = include_str!("../../resources/data/strikeTeams/missionConfig.json");
    RawJson(DEFS)
}

/// GET /striketeams/specializations
pub async fn get_specializations() -> Json<ListWithCount<StrikeTeamSpecialization>> {
    let strike_teams = StrikeTeamDefinitions::get();

    Json(ListWithCount::new(&strike_teams.specializations))
}

/// GET /striketeams/equipment
pub async fn get_equipment() -> Json<ListWithCount<StrikeTeamEquipment>> {
    let strike_teams = StrikeTeamDefinitions::get();
    Json(ListWithCount::new(&strike_teams.equipment))
}

/// POST /striketeams/:id/equipment/:name?currency=MissionCurrency
pub async fn purchase_equipment(
    Auth(user): Auth,
    Query(query): Query<PurchaseQuery>,
    Path((id, name)): Path<(StrikeTeamId, String)>,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<PurchaseResponse> {
    let strike_teams = StrikeTeamDefinitions::get();

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

    // TODO: Randomize outcome

    static DEFS: &str = include_str!("../../resources/data/strikeTeams/placeholderResolve.json");
    RawJson(DEFS)
}

/// POST /striketeams/:id/mission/:id
///
/// Obtain the details about a specific strike team mission
pub async fn get_mission(Path((id, mission_id)): Path<(Uuid, Uuid)>) -> RawJson {
    debug!("Strike team get mission : {} {}", id, mission_id);

    // TODO: Randomize outcome

    static DEFS: &str = include_str!("../../resources/data/strikeTeams/missionSpecific.json");
    RawJson(DEFS)
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
    let strike_team_cost = *StrikeTeamDefinitions::STRIKE_TEAM_COSTS
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
    let next_purchase_cost = StrikeTeamDefinitions::STRIKE_TEAM_COSTS
        .get(strike_teams + 1)
        .copied();

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost,
    }))
}
