//! Strike team related logic
//!
//! Every mission has one "Enemy" trait and two "Mission" traits
//!
//! The collection of strike team missions available are the same for *every* player
//! and are rotated

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use uuid::{uuid, Uuid};

use super::{
    challenges::CurrencyReward,
    i18n::{I18nDesc, I18nDescription, I18nName},
    items::{ItemDefinition, ItemName},
    shared::CustomAttributes,
};
use crate::utils::ImStr;

/// Type alias for a [ImStr] representing a [MissionTag::name]
pub type MissionTagName = ImStr;

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

impl StrikeTeamTraits {}

/// Represents a trait a strike team can have, can be either
/// a positive or negative trait
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
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

impl MissionTypeDescriptor {
    pub fn normal() -> Self {
        Self {
            name: uuid!("39b9880a-ce11-4be3-a3e7-728763b48614"),
            i18n_name: I18nName::new(12028), /* "Normal" */
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
    pub item_definitions: Vec<ItemDefinition>,
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