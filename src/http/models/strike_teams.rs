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
    pub team: StrikeTeamWithMission,
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
    Completed,
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
    pub custom_attributes: MissionDescriptorAttr,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionDescriptorAttr {
    pub icon: Option<String>,
    pub selector_icon: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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

static KNOWN_MISSION_TRAIT_NAMES: &[(u32, &str)] = &[
    (135549, "Alien Presence"),
    (135550, "No Room for Error"),
    (135551, "Extraction"),
    (135552, "We Need a Hero"),
    (135553, "Assault"),
    (135554, "Nighttime Mission"),
    (135555, "High-Risk, High-Reward"),
    (135556, "Key Intelligence Component"),
    (135557, "Poor Weather Conditions"),
    (135558, "Hostage Situation"),
    (135559, "Silent and Deadly"),
    (135560, "Bribe Attempt"),
    (135561, "Scary"),
    (135562, "Enemies Everywhere"),
];

static KNOWN_TRAIT_NAMES: &[(u32, &str)] = &[
    (153269, "Careless"),
    (153270, "Berserker"),
    (153271, "Poor Intelligence"),
    (153272, "Ill-Prepared"),
    (153273, "Skirmisher"),
    (153274, "Cowardly"),
    (153275, "Virtuous"),
    (153276, "Injured Teammate"),
    (153277, "Shell-Shocked"),
    (153278, "Tough"),
    (153279, "Elite"),
    (153280, "Grizzled Veteran"),
    (153281, "Rugged"),
    (153282, "Heroic"),
    (153283, "Stealthy"),
    (153284, "Unlucky"),
    (153285, "Nighttime Operator"),
    (153286, "Reluctant Soldier"),
    (153287, "Precise"),
    (153288, "Fragile"),
    (153289, "Hero Complex"),
    (153290, "Hostage Rescue Specialist"),
    (153291, "Night Blindness"),
    (153292, "Timid"),
    (153293, "Daring"),
    (153294, "Tactician"),
    (153295, "Corruptible"),
    (153296, "Lucky"),
    (153297, "Fearless"),
    (153298, "Low on Supplies"),
    (153299, "Raucous"),
    (153300, "Slow Reflexes"),
    (153301, "Xenophobe"),
    (153302, "Bloodthirsty"),
];

#[rustfmt::skip]
static KNOWN_TRAIT_DESCS: &[(u32, &str)] = &[
    (216463,"+10 to Effectiveness with Key Intelligence Component"),
    (216462,"-10 to Effectiveness with Bribe Attempt"),
    (216461,"+10 to Effectiveness during Poor Weather Conditions"),
    (216460,"+10 to Effectiveness with We Need a Hero"),
    (216427,"-10 to Effectiveness when Silent and Deadly"),
    (216429,"-10 to Effectiveness with We Need a Hero"),
    (216431,"-10 to Effectiveness with Scary"),
    (216432,"+10 to Effectiveness with No Room for Error"),
    (216433,"+10 to Effectiveness during Defense"),
    (216436,"+10 to Effectiveness during A Hostage Situation"),
    (216437,"+10 to Effectiveness during Assault"),
    (216438,"+10 to Effectiveness with Enemies Everywhere"),
    (216439,"-10 to Effectiveness during Assault"),
    (216440,"+10 to Effectiveness with High-Risk, High-Reward"),
    (216441,"-10 to Effectiveness with High-Risk, High-Reward"),
    (216442,"+10 to Effectiveness when Silent and Deadly"),
    (216443,"-10 to Effectiveness during Poor Weather Conditions"),
    (216444,"-10 to Effectiveness during Extraction"),
    (216445,"-10 to Effectiveness during Nighttime Missions"),
    (216446,"+5 to Effectiveness"),
    (216448,"-10 to Effectiveness with Alien Presence"),
    (216449,"+10 to Effectiveness with Bribe Attempt"),
    (216450,"-10 to Effectiveness with No Room for Error"),
    (216451,"-10 to Effectiveness with Key Intelligence Component"),
    (216452,"-10 to Effectiveness during A Hostage Situation"),
    (216453,"-5 to Effectiveness"),
    (216454,"+10 to Effectiveness with Alien Presence"),
    (216455,"+10 to Effectiveness with Scary"),
    (216456,"+10 to Effectiveness during Extraction"),
    (216457,"+10 to Effectiveness during Nighttime Missions"),
    (216458,"-10 to Effectiveness with Enemies Everywhere"),
    (216459,"-10 to Effectiveness during Defense")
];
