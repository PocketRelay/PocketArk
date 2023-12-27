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

// New missions are posted every four hours, starting at midnight, Eastern Standard Time (-5:00 UTC).

impl Mission {
    pub fn random(rng: &mut StdRng, service: &'static StrikeTeamDefinitions) -> Self {
        let name = Uuid::new_v4();
        // TODO: Filter descriptors based on what they apply to
        let descriptor = service
            .mission_descriptors
            .choose(rng)
            .expect("Failed to select mission descriptor")
            .clone();

        let mission_type = MissionType::normal();

        let access_choices = [
            MissionAccessibility::Any,
            MissionAccessibility::MultiPlayer,
            MissionAccessibility::SinglePlayer,
        ];

        // TODO: Randomly decide whether mission should be apex
        let accessibility: MissionAccessibility = access_choices
            .choose_weighted(rng, |access| access.weight())
            .copied()
            .expect("Failed to choose accessibility");

        // TODO: Waves only need to be specified for custom missions
        let waves = Vec::new();

        let mut tags: Vec<&'static MissionTag> = Vec::with_capacity(3);

        let enemy_tag = service
            .mission_traits
            .enemy
            .choose(rng)
            .expect("Missing enemy tag");
        tags.push(enemy_tag);

        service
            .mission_traits
            .game
            .choose_multiple(rng, 2)
            .for_each(|value| tags.push(value));

        // TODO: Modifiers
        let mut static_modifiers = Vec::new();
        let dynamic_modifiers = Vec::new();

        let diffs = [("bronze", 8), ("silver", 6), ("gold", 2), ("platinum", 1)];
        let levels = [
            "MPGreen", "MPBlack", "MPBlue", "MPGrey", "MPOrange", "MPYellow", "MPAqua", "MPTower",
            "MPHangar",
        ];

        let (difficulty, _) = diffs
            .choose_weighted(rng, |(_, weight)| *weight)
            .expect("Failed to select difficulty");

        let level = levels.choose(rng).expect("Failed to choose level");

        static_modifiers.push(MissionModifier {
            name: "difficulty".to_string(),
            value: difficulty.to_string(),
        });

        static_modifiers.push(MissionModifier {
            name: "enemyType".to_string(),
            value: enemy_tag.name.to_string(),
        });

        static_modifiers.push(MissionModifier {
            name: "level".to_string(),
            value: level.to_string(),
        });

        let rewards = MissionRewards::random(rng, service, &accessibility, difficulty);
        let custom_attributes = CustomAttributes::default();
        // TODO: Custom attrs

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let day = 60 * 60 * 24;

        // TODO: Properly random gen expiry?
        let start_seconds = now;
        let end_seconds = now + day;

        let sp_length_seconds = 4941; // TODO: Randomly decide duration that strike teams take

        Self {
            name,
            descriptor,
            mission_type,
            accessibility,
            waves,
            tags,
            static_modifiers,
            dynamic_modifiers,
            rewards,
            custom_attributes,
            start_seconds,
            end_seconds,
            sp_length_seconds,
        }
    }
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

impl MissionRewards {
    pub fn random(
        rng: &mut StdRng,
        service: &StrikeTeamDefinitions,
        access: &MissionAccessibility,
        difficulty: &str,
    ) -> Self {
        let currency_reward = match access {
            MissionAccessibility::SinglePlayer => CurrencyReward {
                name: CurrencyType::Mission,
                value: 5,
            },
            // Apex mission rewards
            MissionAccessibility::Any | MissionAccessibility::MultiPlayer => CurrencyReward {
                name: CurrencyType::Mission,
                value: 10,
            },
        };

        // TODO: Properly implement
        Self {
            name: Uuid::new_v4(),
            currency_reward,
            mp_item_rewards: HashMap::new(),
            sp_item_rewards: HashMap::new(),
            item_definitions: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Wave {
    pub name: Uuid,
    pub wave_type: WaveType,
    pub custom_attributes: CustomAttributes,
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

#[derive(Debug, Serialize, Clone, Copy)]
pub enum MissionAccessibility {
    #[serde(rename = "Single_Player")]
    SinglePlayer,
    #[serde(rename = "Multi_Player")]
    MultiPlayer,
    #[serde(other)]
    Any,
}

impl MissionAccessibility {
    pub fn weight(&self) -> u8 {
        match self {
            MissionAccessibility::SinglePlayer => 6,
            MissionAccessibility::Any => 3,
            MissionAccessibility::MultiPlayer => 1,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissionState {
    PendingResolve,
    Available,
    Completed,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionDescriptor {
    pub name: Uuid,
    #[serde(flatten)]
    pub i18n_name: I18nName,

    #[serde(flatten)]
    pub i18n_desc: Option<I18nDesc>,

    #[serde(default)]
    pub custom_attributes: MissionDescriptorAttr,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MissionDescriptorAttr {
    pub icon: Option<String>,
    pub selector_icon: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
