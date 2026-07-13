use anyhow::Context;
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantArray};

/// Different maps that can be used for missions
#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, VariantArray, EnumString, Display,
)]
#[allow(clippy::enum_variant_names)]
pub enum MissionLevel {
    #[serde(rename = "MPGreen")]
    MPGreen,
    #[serde(rename = "MPBlack")]
    MPBlack,
    #[serde(rename = "MPBlue")]
    MPBlue,
    #[serde(rename = "MPGrey")]
    MPGrey,
    #[serde(rename = "MPOrange")]
    MPOrange,
    #[serde(rename = "MPYellow")]
    MPYellow,
    #[serde(rename = "MPAqua")]
    MPAqua,
    #[serde(rename = "MPTower")]
    MPTower,
    #[serde(rename = "MPHangar")]
    MPHangar,
}

impl MissionLevel {
    pub fn random<R>(rng: &mut R) -> anyhow::Result<MissionLevel>
    where
        R: Rng,
    {
        MissionLevel::VARIANTS
            .choose(rng)
            .context("Failed to choose level")
            .copied()
    }
}
