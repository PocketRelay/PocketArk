//! Strike team related logic
//!
//! Every mission has one "Enemy" trait and two "Mission" traits
//!
//! The collection of strike team missions available are the same for *every* player
//! and are rotated

use crate::{
    database::entity::{
        currency::CurrencyType, strike_team_mission::MissionAccessibility, StrikeTeam, User,
    },
    definitions::{
        challenges::CurrencyReward,
        i18n::{I18nDesc, I18nDescription, I18nName},
        items::{ItemDefinition, ItemName},
        level_tables::{LevelTable, LevelTableName, LevelTables, ProgressionXp},
        shared::CustomAttributes,
    },
    utils::ImStr,
};
use anyhow::Context;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use sea_orm::{ConnectionTrait, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::{collections::HashMap, sync::OnceLock};
use uuid::{uuid, Uuid};

use super::items::Items;

/// Type alias for a [ImStr] representing a [MissionTag::name]
pub type MissionTagName = ImStr;

const STRIKE_TEAM_TRAIT_DEFINITIONS: &str = include_str!("../resources/data/strikeTeamTraits.json");
const STRIKE_TEAM_TAG_DEFINITIONS: &str = include_str!("../resources/data/strikeTeamTags.json");
const STRIKE_TEAM_MISSION_DEFINITIONS: &str =
    include_str!("../resources/data/strikeTeamMissions.json");

/// Collection of names that strike teams are randomly named from
///
/// Sourced from "NATO phonetic alphabet"
static STRIKE_TEAM_NAMES: &[&str] = &[
    "Yankee", "Delta", "India", "Echo", "Zulu", "Charlie", "Whiskey", "Lima", "Bravo", "Sierra",
    "November", "X-Ray", "Golf", "Alpha", "Romeo", "Kilo", "Tango", "Quebec", "Foxtrot", "Papa",
    "Mike", "Oscar", "Juliet", "Uniform", "Victor", "Hotel",
];

/// Name of the [LevelTable] used for leveling strike teams
static STRIKE_TEAM_LEVEL_TABLE: LevelTableName = uuid!("5e6f7542-7309-9367-8437-fe83678e5c28");

/// Collection of strike team icons and their associated internal
/// team name
static STRIKE_TEAM_ICON_SETS: &[(&str, &str)] = &[
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

pub const MAX_STRIKE_TEAMS: usize = 6;
pub static STRIKE_TEAM_COSTS: [u32; MAX_STRIKE_TEAMS] = [0, 40, 80, 120, 160, 200];

pub struct StrikeTeams {
    pub traits: StrikeTeamTraits,
    pub tags: MissionTags,
    pub missions: MissionDefinitions,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<StrikeTeams> = OnceLock::new();

impl StrikeTeams {
    /// Gets a static reference to the global [StrikeTeamDefinitions] collection
    pub fn get() -> &'static StrikeTeams {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let traits: StrikeTeamTraits = serde_json::from_str(STRIKE_TEAM_TRAIT_DEFINITIONS)
            .context("Failed to load strike team traits")?;
        let tags: MissionTags = serde_json::from_str(STRIKE_TEAM_TAG_DEFINITIONS)
            .context("Failed to load strike team mission tags")?;
        let missions: MissionDefinitions = serde_json::from_str(STRIKE_TEAM_MISSION_DEFINITIONS)
            .context("Failed to load strike team mission definitions")?;

        Ok(Self {
            traits,
            tags,
            missions,
        })
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

/// Creates a new strike team for the provided user
pub async fn create_user_strike_team<C>(db: &C, user: &User) -> anyhow::Result<StrikeTeam>
where
    C: ConnectionTrait + Send,
{
    // Generate random strike team data
    let mut rng = StdRng::from_entropy();
    let strike_team_data = random_strike_team(&mut rng).context("Failed to create strike team")?;

    // Create the strike team
    let team = StrikeTeam::create(db, &user, strike_team_data).await?;
    Ok(team)
}

pub fn random_strike_team<R>(rng: &mut R) -> anyhow::Result<StrikeTeamData>
where
    R: Rng,
{
    let strike_teams = StrikeTeams::get();

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
    let positive_trait = strike_teams.traits.random_postitive(rng)?;

    Ok(StrikeTeamData {
        name,
        icon,
        level,
        xp,
        positive_trait,
    })
}

/// Type alias for the name of a strike team
pub type StrikeTeamName = String;

/// Chooses a random strike team name from [STRIKE_TEAM_NAMES]
fn random_team_name<R>(rng: &mut R) -> anyhow::Result<StrikeTeamName>
where
    R: Rng,
{
    STRIKE_TEAM_NAMES
        .choose(rng)
        .context("Failed to choose name")
        .map(|value| value.to_string())
}

/// Icon that the a strike team can use
///
/// For reference: https://masseffectandromeda.fandom.com/wiki/Strike_team#Team_composition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamIcon {
    /// Name of the icon
    pub name: ImStr,
    /// Icon image path
    pub image: ImStr,
}

impl StrikeTeamIcon {
    /// Choose a random strike team icon
    fn random<R>(rng: &mut R) -> anyhow::Result<Self>
    where
        R: Rng,
    {
        STRIKE_TEAM_ICON_SETS
            .choose(rng)
            .context("Failed to choose icon")
            .map(|(name, image)| Self {
                name: ImStr::from(*name),
                image: ImStr::from(*image),
            })
    }
}

/// Collection of mission tags, split based on their different types
#[derive(Debug, Serialize, Deserialize)]
pub struct MissionTags {
    /// Mission tags for enemies (To choose which enemy is used)
    pub enemy: Vec<MissionTag>,
    /// Mission specific tags (To chooes various factors about the mission i.e night-time)
    pub mission: Vec<MissionTag>,
}

/// Collection of traits based on a positive or negative factor
#[derive(Debug, Serialize, Deserialize)]
pub struct StrikeTeamTraits {
    /// Collection of positive traits
    pub positive: Box<[StrikeTeamTrait]>,
    /// Collection of negative traits
    pub negative: Box<[StrikeTeamTrait]>,
}

impl StrikeTeamTraits {
    /// Choose a random positive trait
    fn random_postitive<R>(&self, rng: &mut R) -> anyhow::Result<StrikeTeamTrait>
    where
        R: Rng,
    {
        self.positive
            .choose(rng)
            .context("Failed to choose trait")
            .cloned()
    }

    /// Finds a [StrikeTeamTrait] by a specific mission `tag` and uses
    /// `positive` to determine whether the trait must be positive or negative
    fn by_mission_tag(&self, tag: &MissionTagName, positive: bool) -> Option<&StrikeTeamTrait> {
        let list: &[StrikeTeamTrait] = match positive {
            true => &self.positive,
            false => &self.negative,
        };

        list.iter().find(|value| {
            value
                .tag
                .as_ref()
                .is_some_and(|value_tag| value_tag.eq(tag))
        })
    }
}

/// Represents a trait a strike team can have, can be either
/// a positive or negative trait
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikeTeamTrait {
    /// Same as the `i18nName` field
    pub name: ImStr,
    /// The tag this trait is based on, for general traits
    /// this is not set
    pub tag: Option<MissionTagName>,
    /// The effectiveness of the trait, positive values from
    /// improved effectiveness and negative for worsened
    pub effectiveness: i8,

    /// Localized name of the trait
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized description of the trait
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MissionDifficulty {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Collection of mission definitions
#[derive(Deserialize)]
pub struct MissionDefinitions {
    /// Collection of missions for each difficulty level
    pub difficulty: HashMap<MissionDifficulty, MissionTypeGroup>,
    /// Collection of special missions that aren't given by random
    pub special: Vec<MissionDefinition>,
}

/// Mission definitions grouped based on the
/// different types (standard and apex)
#[derive(Deserialize)]
pub struct MissionTypeGroup {
    pub standard: Vec<MissionDefinition>,
    pub apex: Vec<MissionDefinition>,
}

/// Definition for a mission
#[derive(Deserialize)]
pub struct MissionDefinition {
    /// The mission descriptor
    pub descriptor: MissionDescriptor,
    /// The mission accessibility
    pub accessibility: MissionAccessibility,
    /// Optional collection of waves for custom missions
    #[serde(default)]
    pub waves: Option<Vec<MissionWave>>,
    /// Optional overriden mission rewards
    #[serde(default)]
    pub rewards: Option<MissionRewards>,
}

/// Represents a tag that a mission can have associated with it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionTag {
    /// Name of the mission tag
    pub name: MissionTagName,
    /// Localized name of the tag
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized description of the tag (Appears unused)
    #[serde(flatten)]
    pub i18n_desc: I18nDesc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionModifier {
    /// The name of the modifier ("difficulty", "enemyType", "level", etc)
    pub name: ImStr,
    /// The value of the modifier
    pub value: ImStr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct MissionType {
    /// The unique ID name for the type
    pub name: Uuid,
    /// Descriptor for the mission
    pub descriptor: MissionTypeDescriptor,
    /// Whether the mission gives currency rewards
    pub give_currency: bool,
    /// Whether the mission gives XP
    pub give_xp: bool,
}

impl Default for MissionType {
    fn default() -> Self {
        Self {
            name: uuid!("1cedd0c2-652b-d879-d8c9-0ff8b1b0bf9c"),
            descriptor: Default::default(),
            give_currency: true,
            give_xp: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct MissionTypeDescriptor {
    pub name: Uuid,
    #[serde(flatten)]
    pub i18n_name: I18nName,

    #[serde(flatten)]
    pub i18n_desc: Option<I18nDesc>,

    pub custom_attributes: CustomAttributes,
}

impl Default for MissionTypeDescriptor {
    fn default() -> Self {
        Self {
            name: uuid!("39b9880a-ce11-4be3-a3e7-728763b48614"),
            i18n_name: I18nName::new(12028 /* "Normal" */),
            i18n_desc: None,
            custom_attributes: Default::default(),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct MissionDescriptor {
    /// Unique ID for the mission descriptor
    pub name: Uuid,

    /// Attributes for the mission descriptor
    /// contains the icons for the descriptor
    #[serde(default)]
    pub custom_attributes: CustomAttributes,

    /// Localized name for the mission type
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized description for the mission type
    #[serde(flatten)]
    pub i18n_desc: Option<I18nDesc>,
}

pub type MissionRewardsId = Uuid;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct MissionRewards {
    /// Unique ID for the rewards collection
    pub name: MissionRewardsId,
    /// Currency rewards from the mission
    pub currency_reward: CurrencyReward,
    /// Multiplayer items earned from the mission
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub mp_item_rewards: Vec<(ItemName, u32)>,
    /// Singleplayer items earned from the mission
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub sp_item_rewards: Vec<(ItemName, u32)>,
    /// Definitions of the items that should be earned
    #[serde(default)]
    pub item_definitions: Vec<ItemDefinition>,
}

impl MissionRewards {
    pub fn default(difficulty: MissionDifficulty, accessibility: MissionAccessibility) -> Self {
        let mut currency_reward = CurrencyReward {
            name: CurrencyType::Mission,
            value: 0,
        };

        let mut mp_item_rewards: Vec<(ItemName, u32)> = Vec::new();
        let mut sp_item_rewards: Vec<(ItemName, u32)> = Vec::new();

        match accessibility {
            MissionAccessibility::Any | MissionAccessibility::MultiPlayer => {
                // Platinum gives 15 mission currency instead of 10
                if let MissionDifficulty::Platinum = difficulty {
                    currency_reward.value = 15
                } else {
                    currency_reward.value = 10
                }

                match difficulty {
                    MissionDifficulty::Bronze => {
                        // "Bronze Item Loot Box"
                        sp_item_rewards.push((uuid!("14d5e5ba-dbb5-4336-ad07-607eb39409bb"), 1));
                        // "Research Data Loot Box"
                        sp_item_rewards.push((uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1));
                    }
                    MissionDifficulty::Silver => {
                        // "Silver Item Loot Box"
                        sp_item_rewards.push((uuid!("a7d46d7a-1f42-4eac-b106-c2fb96aa3e7a"), 1));
                        // "Research Data Loot Box"
                        sp_item_rewards.push((uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1));
                    }
                    MissionDifficulty::Gold | MissionDifficulty::Platinum => {
                        // "Gold Item Loot Box"
                        sp_item_rewards.push((uuid!("58383d3f-d74d-4518-b27e-988f56ade54c"), 1));
                        // "Research Data Loot Box"
                        sp_item_rewards.push((uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1));
                    }
                };
            }
            MissionAccessibility::SinglePlayer => {
                // Strike team missions give 5 mission currency
                currency_reward.value = 5;

                match difficulty {
                    MissionDifficulty::Bronze => {
                        // "Bronze Credit Loot Box"
                        sp_item_rewards.push((uuid!("e300500e-885e-4ee5-bbdc-f706b30b362a"), 1));
                        // "Bronze Material Loot Box"
                        sp_item_rewards.push((uuid!("1440d464-0245-49f9-8533-4930b9283d78"), 1));
                    }
                    MissionDifficulty::Silver => {
                        // "Silver Credit Loot Box"
                        sp_item_rewards.push((uuid!("e4556800-5eef-d487-182f-5044f0f2d534"), 1));
                        // "Silver Material Loot Box"
                        sp_item_rewards.push((uuid!("004f85aa-f7ac-4262-8109-e7e7d6d94bd5"), 1));
                    }
                    MissionDifficulty::Gold => {
                        // "Gold Credit Loot Box"
                        sp_item_rewards.push((uuid!("9860be4d-b3b2-445f-aa7d-1728fc163ddb"), 1));
                        // "Silver Material Loot Box"
                        sp_item_rewards.push((uuid!("61d3f563-ad29-4f97-9c80-71c72549a5fe"), 1));
                    }
                    // Platnum mission should *never* be single player (Strike team) missions
                    MissionDifficulty::Platinum => {}
                };
            }
        };

        let items = Items::get();

        let item_definitions = mp_item_rewards
            .iter()
            .chain(sp_item_rewards.iter())
            .filter_map(|(item, _)| items.by_name(item))
            .cloned()
            .collect();

        Self {
            name: Uuid::new_v4(),
            currency_reward,
            mp_item_rewards,
            sp_item_rewards,
            item_definitions,
        }
    }
}

pub type MissionWaveName = Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionWave {
    /// Unique ID for the wave
    pub name: MissionWaveName,
    /// The type of wave
    pub wave_type: WaveType,
    /// Custom attributes associated with the wave
    pub custom_attributes: CustomAttributes,
}

/// Types of [MissionWave]s
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WaveType {
    /// Wave has an objective
    #[serde(rename = "WaveType_Objective")]
    Objective,
    /// Wave is just enemies
    #[serde(rename = "WaveType_Hoard")]
    Hoard,
    /// Wave is the extraction
    #[serde(rename = "WaveType_Extraction")]
    Extraction,
}
