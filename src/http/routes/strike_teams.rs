use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    Json,
};
use hyper::StatusCode;
use log::debug;
use uuid::Uuid;

use crate::{
    database::entity::{Currency, StrikeTeam},
    http::{
        middleware::user::Auth,
        models::{
            strike_teams::{PurchaseQuery, PurchaseResponse, StrikeTeamsList, StrikeTeamsResponse},
            HttpError, HttpResult, ListWithCount, RawJson,
        },
    },
    services::strike_teams::{
        StrikeTeamEquipment, StrikeTeamService, StrikeTeamSpecialization, StrikeTeamWithMission,
    },
    state::App,
};

/// GET /striketeams
pub async fn get(Auth(user): Auth) -> HttpResult<StrikeTeamsResponse> {
    let db = App::database();
    let strike_teams: Vec<StrikeTeam> = StrikeTeam::get_by_user(db, &user).await?;

    // TODO: Load current missions
    let teams: Vec<StrikeTeamWithMission> = strike_teams
        .into_iter()
        .map(|team| StrikeTeamWithMission {
            mission: None,
            team,
        })
        .collect();

    let mut next_purchase_costs: HashMap<String, u32> = HashMap::new();

    // Get new cost
    let strike_team_cost = StrikeTeamService::STRIKE_TEAM_COSTS
        .get(teams.len())
        .copied();
    if let Some(strike_team_cost) = strike_team_cost {
        next_purchase_costs.insert("MissionCurrency".to_string(), strike_team_cost);
    }

    Ok(Json(StrikeTeamsResponse {
        teams: StrikeTeamsList {
            total_count: teams.len(),
            cap: StrikeTeamService::MAX_STRIKE_TEAMS,
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
    let services = App::services();
    Json(ListWithCount::new(&services.strike_teams.specializations))
}

/// GET /striketeams/equipment
pub async fn get_equipment() -> Json<ListWithCount<StrikeTeamEquipment>> {
    let services = App::services();
    Json(ListWithCount::new(&services.strike_teams.equipment))
}

/// POST /striketeams/:id/equipment/:name?currency=MissionCurrency
pub async fn purchase_equipment(
    Auth(user): Auth,
    Query(query): Query<PurchaseQuery>,
    Path((id, name)): Path<(Uuid, String)>,
) -> HttpResult<PurchaseResponse> {
    let db = App::database();

    let currency = Currency::get_type_from_user(db, &user, &query.currency)
        .await?
        .ok_or(HttpError::new(
            "Currency balance cannot be less than 0.",
            StatusCode::CONFLICT,
        ))?;

    let team = StrikeTeam::get_by_id(db, &user, id)
        .await?
        .ok_or(HttpError::new(
            "Strike team doesn't exist",
            StatusCode::NOT_FOUND,
        ))?;

    // TODO: If on mission respond with 409 Conflict Team on mission

    let services = App::services();
    let equipment = services
        .strike_teams
        .equipment
        .iter()
        .find(|equip| equip.name.eq(&name))
        .ok_or(HttpError::new(
            "Unknown equipment item",
            StatusCode::NOT_FOUND,
        ))?;

    let equipment_cost = equipment
        .cost_by_currency
        .get(&currency.name)
        .copied()
        .ok_or(HttpError::new("Invalid currency", StatusCode::CONFLICT))?;

    // Cannot afford
    if currency.balance < equipment_cost {
        return Err(HttpError::new(
            "Currency balance cannot be less than 0.",
            StatusCode::CONFLICT,
        ));
    }

    // TODO: Transaction to revert incase equipment setting fails

    // Consume currency
    let currency_balance = currency.consume(db, equipment_cost).await?;
    let team = team.set_equipment(db, Some(equipment.clone())).await?;

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
pub async fn retire(Auth(user): Auth, Path(id): Path<Uuid>) -> Result<(), HttpError> {
    debug!("Strike team retire: {}", id);
    let db = App::database();
    let team = StrikeTeam::get_by_id(db, &user, id)
        .await?
        .ok_or(HttpError::new(
            "Strike team doesn't exist",
            StatusCode::NOT_FOUND,
        ))?;

    team.delete(db).await?;

    Ok(())
}

/// POST /striketeams/purchase?currency=MissionCurrency
pub async fn purchase(Auth(user): Auth) -> HttpResult<PurchaseResponse> {
    let db = App::database();

    let strike_teams = StrikeTeam::get_user_count(db, &user).await? as usize;

    // Get new cost
    let strike_team_cost = StrikeTeamService::STRIKE_TEAM_COSTS
        .get(strike_teams)
        .copied()
        .ok_or(HttpError::new(
            "Maximum number of strike teams reached",
            StatusCode::CONFLICT,
        ))?;

    let currency = Currency::get_type_from_user(db, &user, "MissionCurrency")
        .await?
        .ok_or(HttpError::new(
            "Currency balance cannot be less than 0.",
            StatusCode::CONFLICT,
        ))?;

    // Cannot afford
    if currency.balance < strike_team_cost {
        return Err(HttpError::new(
            "Currency balance cannot be less than 0.",
            StatusCode::CONFLICT,
        ));
    }

    // TODO: Transaction to revert incase strike team creation fails

    // Consume currency
    let currency_balance = currency.consume(db, strike_team_cost).await?;
    let team = StrikeTeam::create_default(db, &user).await?;

    // Get new cost
    let next_purchase_cost = StrikeTeamService::STRIKE_TEAM_COSTS
        .get(strike_teams + 2)
        .copied();

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost,
    }))
}
