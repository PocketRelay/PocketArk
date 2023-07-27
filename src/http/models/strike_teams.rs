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

static KNOWN_MISSION_NAMES: &[(u32, &str)] = &[
    (216304, "PROTECT SENSITIVE EQUIPMENT"),
    (216305, "IDENTIFY THE SPY"),
    (216913, "OBTAIN ENEMY SHIP LOCATIONS"),
    (216914, "ELIMINATE THE TRAITOR"),
    (216915, "LOCATE NEW RESOURCES"),
    (216916, "RECOVER STOLEN POWER GENERATORS"),
    (216917, "DISTRACT ENEMY FORCES"),
    (216918, "DETERMINE ENEMY STRENGTH"),
    (216919, "OBTAIN PROTOTYPE SCHEMATICS"),
    (216920, "ELIMINATE ENEMY SABOTEURS"),
    (216921, "ACQUIRE ENEMY RESEARCH"),
    (216922, "INVESTIGATE FACILITY"),
    (216923, "INVESTIGATE WEAPON REPORTS"),
    (216924, "ESTABLISH ANTI-SHIP BATTERIES"),
    (216925, "SABOTAGE ENEMY OPERATIONS"),
    (216926, "OBTAIN DEAD-DROP DATA"),
    (216927, "DELAY SHIP LAUNCHES"),
    (216928, "OBTAIN MINING LOCATIONS"),
    (216929, "UNCOVER ENEMY PLANS"),
    (216930, "RETAKE AN OCCUPIED FACILITY"),
    (216931, "MEASURE ENEMY RESPONSE"),
    (216932, "ESTABLISH A LISTENING POST"),
    (216933, "INVESTIGATE UNKNOWN SIGNALS"),
    (216934, "ESTABLISH A SUPPLY DEPOT"),
    (216935, "PUNISH KILLERS"),
    (216936, "PERFORM A SURGICAL STRIKE"),
    (216937, "RECOVER STOLEN HABITATION DATA"),
    (216938, "SABOTAGE WEAPONS SHIPMENT"),
    (216939, "IDENTIFY ENEMY LOADOUTS"),
    (216940, "OBTAIN TECH PROTOTYPE"),
    (216941, "INVESTIGATE A POSSIBLE TRAP"),
    (216942, "OBTAIN ENCRYPTION KEY"),
    (216943, "FALSE FLAG OPERATION"),
    (216944, "DESTROY ENEMY WARSHIPS"),
];

static KNOWN_MISSION_DESC: &[(u32, &str)] = &[
    (216877,"Hostile forces are planning to launch several ships that could threaten our operations. Keep those ships from launching until our forces are ready to fight them."),
    (216878,"We need the enemy to pay attention to something other than our planned activity. Hit them as loudly as possible to draw their attention so that we can deploy forces covertly."),
    (216879,"To utilize a potential site, we need to secure it against enemy bombardment. Secure the facility so we can establish anti-ship batteries."),
    (216880,"We've heard rumors of enemy forces moving weapons through the area. Hit the enemy base and find out what they're moving, and where."),
    (216881,"An agent embedded among enemy forces left us vital intel at a dead drop. Locate and retrieve it before enemy troops discover the drop."),
    (216882,"Our planned surveillance operations are blocked by the presence of an enemy base near our ideal site. Take out that base and give us a hiding place."),
    (216883,"We have reason to believe one of our people is giving intel to the enemy. Hit an enemy communication center and get us a name."),
    (216884,"Rumors are circulating about schematics for a new prototype based on Remnant tech, and we need to investigate. Get into the targeted research facility and find out if the rumors are true."),
    (216885,"We received reports of a new facility, and it's not clear who is running or what their goal is. Investigate and determine if the facility presents a threat."),
    (216886,"The enemy has been quiet lately, and it's likely that they're planning something. Hit one of their listening posts and pull any intel on upcoming plans."),
    (216887,"We've got reports of valuable mineral resources in an area with hostile activity. Find out if the enemy is sitting on anything we want, and make them think twice about how much they want it."),
    (216888,"Hostile forces appear to be planning an attack, but it may be a feint to distract us. Harry their defenses and get an accurate estimation of their strength."),
    (216889,"We've been unable to break the enemy's recent encryption protocols. Hit their listening post and obtain an encryption key before they can destroy their own data."),
    (216890,"An enemy group is performing medical research that’s yielded some promising and dangerous results. Infiltrate the facility and get us that research."),
    (216891,"Hostile forces are resisting our attempts to gain a foothold in the area. Hit their base and sabotage their operations while we prepare an offensive."),
    (216892,"Hostile forces are set to receive a major weapons shipment. Take out the enemy team and stop that weapons shipment from reaching its destination."),
    (216893,"We recently lost several supply stations to enemy saboteurs. Unfortunately for them, we located their base. Get in there and take them down."),
    (216894,"Enemies have been catching us by surprise, and we've been unable to locate their strike ship. Hit their resupply center and find out where that ship is."),
    (216895,"We've heard reports of a valuable new tech prototype in the area. Investigate the facility and obtain that prototype."),
    (216896,"We need to determine whether hostile forces intend to capture one of our installations or destroy it entirely. Hit their supply base and identify the loadout of their heavy weapons."),
    (216897,"The enemy is in possession of detailed mining locations. Hit their operations center and get us those sites."),
    (216898,"We've got intel about bad blood between multiple hostile groups in the area, and we'd like to encourage that. Take out one group and leave evidence to incriminate another."),
    (216899,"Hostile forces stole power generators that one of our colony sites depends on for survival. Hit them hard and get our allies the supplies they need to keep the lights on."),
    (216900,"Enemy forces stole our data on possible habitation sites, then corrupted our local copies. Hit their site and get us that data back."),
    (216901,"One of our key facilities was taken by enemy forces, costing us supplies and intel. Retake the facility before the enemy can lock it down."),
    (216902,"An enemy facility is currently blocking our strategic plans. We need it destroyed quickly and quietly."),
    (216903,"One of our people turned traitor and is working with hostile forces. Hunt him down and take him out before he does any more damage."),
    (216904,"Enemy warships have been wreaking havoc in the area. Ground those ships before they can do any more damage."),
    (216905,"Unknown signals are interfering with our communications in the area. Check the origin coordinates, find the source, and shut it down if hostile."),
    (216906,"We're planning a large operation and need to know the response time and strength of our targets. Poke the hornet's nest and report on their reactions."),
    (216907,"We've picked up some leaked intel that looks too good to be true. Investigate the lead and be ready for an enemy ambush."),
    (216908,"Our efforts have been stymied by a supply shortage. We need you to take an enemy facility and convert it to a supply depot we can use."),
    (216909,"Enemy forces recently targeted a small unarmed ship and killed all aboard. Send them a message about what happens to people who kill unarmed civilians."),
];

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
    (211948, "Remnant Hysteria"),
    (211949, "Outlaw Hysteria"),
    (211950, "Kett Specialist"),
    (211951, "Remnant Specialist"),
    (211952, "Kett Hysteria"),
    (211953, "Outlaw Specialist"),
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
    (216464, "+10 to Effectiveness against outlaws"),
    (216463,"+10 to Effectiveness with Key Intelligence Component"),
    (216462,"-10 to Effectiveness with Bribe Attempt"),
    (216461,"+10 to Effectiveness during Poor Weather Conditions"),
    (216460,"+10 to Effectiveness with We Need a Hero"),
    (216427,"-10 to Effectiveness when Silent and Deadly"),
    (216428,"+10 to Effectiveness against Remnant"),
    (216429,"-10 to Effectiveness with We Need a Hero"),
    (216430,"-10 to Effectiveness against Remnant"),
    (216431,"-10 to Effectiveness with Scary"),
    (216432,"+10 to Effectiveness with No Room for Error"),
    (216433,"+10 to Effectiveness during Defense"),
    (216434,"+10 to Effectiveness against kett"),
    (216435,"-10 to Effectiveness against outlaws"),
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
    (216447,"-10 to Effectiveness against kett"),
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
