//! Strike team related logic
//!
//! Every mission has one "Enemy" trait and two "Mission" traits
//!
//! The collection of strike team missions available are the same for *every* player
//! and are rotated

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::i18n::{I18nDesc, I18nDescription, I18nName};
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
#[derive(Debug, Serialize, Deserialize)]
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
