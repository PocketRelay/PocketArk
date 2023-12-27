#![allow(unused)]
use crate::{
    database::entity::{currency::CurrencyType, StrikeTeam},
    definitions::{
        challenges::CurrencyReward,
        i18n::{I18nDesc, I18nDescription, I18nName},
        items::ItemDefinition,
    },
    http::models::mission::MissionModifier,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use rand::{rngs::StdRng, seq::SliceRandom};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::{uuid, Uuid};

use super::shared::CustomAttributes;

const EQUIPMENT_DEFINITIONS: &str = include_str!("../resources/data/strikeTeams/equipment.json");
const SPECIALIZATION_DEFINITIONS: &str =
    include_str!("../resources/data/strikeTeams/specializations.json");
const MISSION_DESCRIPTORS: &str =
    include_str!("../resources/data/strikeTeams/missionDescriptors.json");
const MISSION_TRAITS: &str = include_str!("../resources/data/strikeTeams/missionTraits.json");

#[derive(Debug, Deserialize)]
pub struct MissionTraits {
    pub enemy: Vec<MissionTag>,
    pub game: Vec<MissionTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionTag {
    pub name: String,

    #[serde(flatten)]
    pub i18n_name: I18nName,
    #[serde(flatten)]
    pub i18n_desc: I18nDesc,
}

pub struct StrikeTeamDefinitions {
    pub equipment: Vec<StrikeTeamEquipment>,
    pub specializations: Vec<StrikeTeamSpecialization>,
    pub mission_descriptors: Vec<MissionDescriptor>,
    pub mission_traits: MissionTraits,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<StrikeTeamDefinitions> = OnceLock::new();

impl StrikeTeamDefinitions {
    pub const STRIKE_TEAM_COSTS: &'static [u32] = &[0, 40, 80, 120, 160, 200];
    pub const MAX_STRIKE_TEAMS: usize = Self::STRIKE_TEAM_COSTS.len();

    /// Gets a static reference to the global [StrikeTeamDefinitions] collection
    pub fn get() -> &'static StrikeTeamDefinitions {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let equipment: Vec<StrikeTeamEquipment> = serde_json::from_str(EQUIPMENT_DEFINITIONS)
            .context("Failed to load equipment definitions")?;
        let specializations: Vec<StrikeTeamSpecialization> =
            serde_json::from_str(SPECIALIZATION_DEFINITIONS)
                .context("Failed to load specialization definitions")?;
        let mission_descriptors: Vec<MissionDescriptor> = serde_json::from_str(MISSION_DESCRIPTORS)
            .context("Failed to load mission descriptors")?;
        let mission_traits: MissionTraits =
            serde_json::from_str(MISSION_TRAITS).context("Failed to load mission traits")?;

        Ok(Self {
            equipment,
            specializations,
            mission_descriptors,
            mission_traits,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSuccessRates {
    // ID and name of the strike team
    pub id: Uuid,
    pub name: String,
    // mapping between mission ID and sucess %
    pub mission_success_rate: HashMap<Uuid, f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamWithMission {
    #[serde(flatten)]
    pub team: StrikeTeam,
    pub mission: Option<StrikeTeamMission>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamEquipment {
    pub name: String,

    /// Localized equipment name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized equipment description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,

    pub level_required: u32,
    pub effectiveness: u32,
    pub tags: Option<Vec<String>>,
    pub cost_by_currency: HashMap<CurrencyType, u32>,
    pub custom_attributes: CustomAttributes,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamSpecialization {
    pub name: String,
    /// Localized specialization name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized specialization description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
    pub tag: String,
    pub effectiveness: u32,
    pub custom_attributes: CustomAttributes,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamMission {
    pub name: Uuid,
    pub live_mission: Mission,
    pub finish_time: Option<DateTime<Utc>>,
    pub successful: bool,
    pub earn_negative_trait: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionWithUserData {
    #[serde(flatten)]
    pub mission: Mission,

    pub seen: bool,
    pub user_mission_state: MissionState,
    pub completed: bool,
}
