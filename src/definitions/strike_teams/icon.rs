use crate::utils::ImStr;
use anyhow::Context;
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

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

/// Icon that the a strike team can use
///
/// For reference: https://masseffectandromeda.fandom.com/wiki/Strike_team#Team_composition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamIcon {
    /// Name of the icon
    pub name: ImStr,
    /// Icon image path
    pub image: ImStr,
}

impl StrikeTeamIcon {
    /// Choose a random strike team icon
    pub fn random<R>(rng: &mut R) -> anyhow::Result<Self>
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
