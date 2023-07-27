use std::collections::HashMap;

use argon2::password_hash::rand_core::le;
use chrono::{DateTime, Utc};
use rand::{rngs::StdRng, seq::SliceRandom};
use serde::Serialize;
use serde_json::{Map, Value};
use uuid::{uuid, Uuid};

use crate::{
    services::{
        activity::ActivityResult,
        challenges::CurrencyReward,
        character::{CharacterService, Xp},
        items::ItemDefinition,
    },
    utils::models::LocaleNameWithDesc,
};

use super::mission::MissionModifier;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveMissionResponse {
    pub team: StrikeTeam,
    pub mission_successful: bool,
    pub traits_acquired: Vec<TeamTrait>,
    pub activity_response: ActivityResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamWithMission {
    #[serde(flatten)]
    pub team: StrikeTeam,
    pub mission: Option<StrikeTeamMission>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeam {
    pub id: Uuid,
    pub name: String,
    pub icon: StrikeTeamIcon,
    pub level: u32,
    pub xp: Xp,
    pub positive_traits: Vec<TeamTrait>,
    pub negative_traits: Vec<TeamTrait>,
    pub out_of_date: bool,
}

// Sourced from "NATO phonetic alphabet"
static STRIKE_TEAM_NAMES: &[&str] = &[
    "Yankee", "Delta", "India", "Echo", "Zulu", "Charlie", "Whiskey", "Lima", "Bravo", "Sierra",
    "November", "X-Ray", "Golf", "Alpha", "Romeo", "Kilo", "Tango", "Quebec", "Foxtrot", "Papa",
    "Mike", "Oscar", "Juliet", "Uniform", "Victor", "Hotel",
];

impl StrikeTeam {
    /// The level table used for strike team levels
    const LEVEL_TABLE: Uuid = uuid!("5e6f7542-7309-9367-8437-fe83678e5c28");

    pub fn random(rng: &mut StdRng, character_service: &CharacterService) -> Self {
        let name = STRIKE_TEAM_NAMES
            .choose(rng)
            .expect("Failed to choose strike team name")
            .to_string();
        let level_table = character_service
            .level_table(&Self::LEVEL_TABLE)
            .expect("Missing strike team level table");

        let level = 1;
        let next_xp = level_table
            .get_entry_xp(level)
            .expect("Missing xp requirement for next strike team level");

        let xp = Xp {
            current: 0,
            last: 0,
            next: next_xp,
        };

        let icon = StrikeTeamIcon::random(rng);

        let positive_traits = Vec::new();
        let negative_traits = Vec::new();

        Self {
            id: Uuid::new_v4(),
            name,
            icon,
            level,
            xp,
            positive_traits,
            negative_traits,
            out_of_date: false,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamIcon {
    pub name: String,
    pub image: String,
}

static ICON_SETS: &[(&str, &str)] = &[
    ("icon1", "Team01"),
    ("icon2", "Team02"),
    ("icon3", "Team03"),
    ("icon4", "Team04"),
    ("icon5", "Team05"),
    ("icon6", "Team06"),
    ("icon7", "Team07"),
    ("icon8", "Team08"),
    ("icon9", "Team09"),
    ("icon10", "Team10"),
];

impl StrikeTeamIcon {
    pub fn random(rng: &mut StdRng) -> Self {
        let (name, image) = ICON_SETS.choose(rng).expect("Missing strike team icon set");

        StrikeTeamIcon {
            name: name.to_string(),
            image: image.to_string(),
        }
    }
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub name: Uuid,
    pub descriptor: MissionDescriptor,
    pub mission_type: MissionType,
    pub accessibility: MissionAccessibility,
    pub waves: Vec<Wave>,
    pub tags: Vec<MissionTag>,
    pub static_modifiers: Vec<MissionModifier>,
    pub dynamic_modifiers: Vec<MissionModifier>,
    pub rewards: MissionRewards,
    pub custom_attributes: Map<String, Value>,
    pub start_seconds: u64,
    pub end_seconds: u64,
    pub sp_length_seconds: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionRewards {
    pub name: Uuid,
    pub currency_reward: CurrencyReward,
    pub mp_item_rewards: HashMap<Uuid, u32>,
    pub sp_item_rewards: HashMap<Uuid, u32>,
    pub item_definitions: Vec<&'static ItemDefinition>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionTag {
    pub name: String,
    pub i18n_name: String,
    pub loc_name: String,
    pub i18n_desc: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Wave {
    pub name: Uuid,
    pub wave_type: WaveType,
    pub custom_attributes: Map<String, Value>,
}

#[derive(Debug, Serialize)]
pub enum WaveType {
    #[serde(rename = "WaveType_Objective")]
    Objective,
    #[serde(rename = "WaveType_Hoard")]
    Hoard,
    #[serde(rename = "WaveType_Extraction")]
    Extraction,
}

#[derive(Debug, Serialize)]
pub enum MissionAccessibility {
    #[serde(rename = "Single_Player")]
    SinglePlayer,
    #[serde(rename = "Multi_Player")]
    MultiPlayer,
    #[serde(other)]
    Any,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissionState {
    PendingResolve,
    Available,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionType {
    pub name: Uuid,
    pub descriptor: MissionTypeDescriptor,
    pub give_currency: bool,
    pub give_xp: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionTypeDescriptor {
    pub name: Uuid,
    pub i18n_name: String,
    pub loc_name: Option<String>,
    pub custom_attributes: Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionDescriptor {
    pub name: Uuid,
    pub i18n_name: String,
    pub i18n_desc: Option<String>,
    pub loc_name: Option<String>,
    pub loc_desc: Option<String>,
    pub custom_attributes: Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTrait {
    pub name: String,
    pub tag: String,
    pub effectiveness: u32,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

impl TeamTrait {
    pub fn random_trait() -> Option<Self> {
        todo!("Random trait impl")
    }
}
