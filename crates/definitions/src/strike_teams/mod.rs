//! Strike team related logic
//!
//! Every mission has one "Enemy" trait and two "Mission" traits
//!
//! The collection of strike team missions available are the same for *every* player
//! and are rotated

use std::sync::OnceLock;

use crate::{
    level_tables::{LevelTable, LevelTableName, LevelTables, ProgressionXp},
    strike_teams::{
        equipment::StrikeTeamEquipmentList,
        icon::StrikeTeamIcon,
        mission::{MissionDefinitions, tag::MissionTags},
        name::{StrikeTeamName, random_team_name},
        specialization::StrikeTeamSpecializations,
        traits::{StrikeTeamTrait, StrikeTeamTraits},
    },
};
use anyhow::Context;
use rand::Rng;
use uuid::uuid;

pub mod equipment;
pub mod icon;
pub mod mission;
pub mod name;
pub mod specialization;
pub mod traits;

/// Name of the [LevelTable] used for leveling strike teams
static STRIKE_TEAM_LEVEL_TABLE: LevelTableName = uuid!("5e6f7542-7309-9367-8437-fe83678e5c28");

pub const MAX_STRIKE_TEAMS: usize = 6;
pub static STRIKE_TEAM_COSTS: [u32; MAX_STRIKE_TEAMS] = [0, 40, 80, 120, 160, 200];

pub struct StrikeTeams {
    pub traits: StrikeTeamTraits,
    pub tags: MissionTags,
    pub missions: MissionDefinitions,
    pub equipment: StrikeTeamEquipmentList,
    pub specializations: StrikeTeamSpecializations,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<StrikeTeams> = OnceLock::new();

impl StrikeTeams {
    /// Gets a static reference to the global [StrikeTeamDefinitions] collection
    pub fn get() -> &'static StrikeTeams {
        STORE
            .get()
            .expect("strike teams global must be initialized")
    }

    /// Initialize the strike teams definitions
    pub fn initialize(value: StrikeTeams) {
        _ = STORE.set(value);
    }
}

/// Data used to create a strike team
pub struct StrikeTeamData {
    pub name: StrikeTeamName,
    pub icon: StrikeTeamIcon,
    pub level: u32,
    pub xp: ProgressionXp,
    pub positive_trait: StrikeTeamTrait,
}

pub fn random_strike_team<R>(
    strike_teams: &StrikeTeams,
    rng: &mut R,
) -> anyhow::Result<StrikeTeamData>
where
    R: Rng,
{
    // Default level
    let level: u32 = 1;

    let level_tables = LevelTables::get();

    let name = random_team_name(rng)?;
    let icon = StrikeTeamIcon::random(rng)?;

    let level_table: &LevelTable = level_tables
        .by_name(&STRIKE_TEAM_LEVEL_TABLE)
        .context("Missing strike team level table")?;

    let xp = level_table
        .get_xp_values(level)
        .map(|(previous, current, next)| ProgressionXp {
            last: previous,
            current,
            next,
        })
        .context("Unable to determine initial xp")?;

    // Every team starts with one positive trait
    let positive_trait = strike_teams.traits.random_positive(rng)?;

    Ok(StrikeTeamData {
        name,
        icon,
        level,
        xp,
        positive_trait,
    })
}
