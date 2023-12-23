use crate::{
    database::entity::{currency::CurrencyType, strike_teams::StrikeTeamId, Currency, StrikeTeam},
    definitions::striketeams::{
        StrikeTeamDefinitions, StrikeTeamEquipment, StrikeTeamSpecialization, StrikeTeamWithMission,
    },
    http::{
        middleware::user::Auth,
        models::{
            strike_teams::{
                PurchaseQuery, PurchaseResponse, StrikeTeamError, StrikeTeamsList,
                StrikeTeamsResponse,
            },
            DynHttpError, HttpResult, ListWithCount, RawJson,
        },
    },
};
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use log::debug;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use uuid::Uuid;

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

    let mut next_purchase_costs: HashMap<String, u32> = HashMap::new();

    // Get new cost
    let strike_team_cost = StrikeTeamDefinitions::STRIKE_TEAM_COSTS
        .get(teams.len())
        .copied();
    if let Some(strike_team_cost) = strike_team_cost {
        next_purchase_costs.insert("MissionCurrency".to_string(), strike_team_cost);
    }

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

    let currency = Currency::get(&db, &user, query.currency)
        .await?
        .ok_or(StrikeTeamError::InsufficientCurrency)?;

    let team = StrikeTeam::get_by_id(&db, &user, id)
        .await?
        .ok_or(StrikeTeamError::UnknownTeam)?;

    // TODO: If on mission respond with 409 Conflict Team on mission

    let equipment = strike_teams
        .equipment
        .iter()
        .find(|equip| equip.name.eq(&name))
        .ok_or(StrikeTeamError::UnknownEquipmentItem)?;

    let equipment_cost = equipment
        .cost_by_currency
        .get(&currency.ty)
        .copied()
        .ok_or(StrikeTeamError::InvalidCurrency)?;

    // Cannot afford
    if currency.balance < equipment_cost {
        return Err(StrikeTeamError::InsufficientCurrency.into());
    }

    // TODO: Transaction to revert incase equipment setting fails

    let new_balance = currency.balance - equipment_cost;

    // Consume currency
    let currency_balance = currency.update(&db, new_balance).await?;
    let team = team.set_equipment(&db, Some(equipment.clone())).await?;

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
    let strike_teams = StrikeTeam::get_user_count(&db, &user).await? as usize;

    // Get new cost
    let strike_team_cost = StrikeTeamDefinitions::STRIKE_TEAM_COSTS
        .get(strike_teams)
        .copied()
        .ok_or(StrikeTeamError::MaxTeams)?;

    let currency = Currency::get(&db, &user, CurrencyType::Mission)
        .await?
        .ok_or(StrikeTeamError::InsufficientCurrency)?;

    // Cannot afford
    if currency.balance < strike_team_cost {
        return Err(StrikeTeamError::InsufficientCurrency.into());
    }

    // TODO: Transaction to revert incase strike team creation fails

    let new_balance = currency.balance - strike_team_cost;

    // Consume currency
    let currency_balance = currency.update(&db, new_balance).await?;
    let team = StrikeTeam::create_default(&db, &user).await?;

    // Get new cost
    let next_purchase_cost = StrikeTeamDefinitions::STRIKE_TEAM_COSTS
        .get(strike_teams + 2)
        .copied();

    Ok(Json(PurchaseResponse {
        currency_balance,
        team,
        next_purchase_cost,
    }))
}
