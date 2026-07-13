use anyhow::Context;
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    definitions::{
        i18n::{I18n, I18nDescription, I18nName, Localized},
        strike_teams::mission::tag::MissionTagName,
    },
    utils::ImStr,
};

const STRIKE_TEAM_TRAIT_DEFINITIONS: &str =
    include_str!("../../resources/data/strikeTeamTraits.json");

/// Collection of traits based on a positive or negative factor
#[derive(Debug, Serialize, Deserialize)]
pub struct StrikeTeamTraits {
    /// Collection of positive traits
    pub positive: Box<[StrikeTeamTrait]>,
    /// Collection of negative traits
    pub negative: Box<[StrikeTeamTrait]>,
}

impl StrikeTeamTraits {
    pub fn from_str(s: &str) -> serde_json::Result<StrikeTeamTraits> {
        serde_json::from_str(s)
    }

    pub fn load() -> serde_json::Result<Self> {
        Self::from_str(STRIKE_TEAM_TRAIT_DEFINITIONS)
    }

    /// Choose a random positive trait
    pub fn random_positive<R>(&self, rng: &mut R) -> anyhow::Result<StrikeTeamTrait>
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
    #[allow(unused)]
    pub fn by_mission_tag(&self, tag: &MissionTagName, positive: bool) -> Option<&StrikeTeamTrait> {
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

impl Localized for StrikeTeamTraits {
    fn localize(&mut self, i18n: &I18n) {
        self.positive
            .iter_mut()
            .chain(self.negative.iter_mut())
            .for_each(|value| value.localize(i18n))
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

impl Localized for StrikeTeamTrait {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        self.i18n_description.localize(i18n);
    }
}
