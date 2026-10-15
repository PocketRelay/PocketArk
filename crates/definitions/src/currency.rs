use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, FromRepr};
use thiserror::Error;

#[derive(
    Debug, Clone, FromRepr, EnumIter, Copy, PartialEq, Eq, Hash, Display, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum CurrencyType {
    #[serde(rename = "MTXCurrency")]
    #[strum(serialize = "MTXCurrency")]
    Mtx = 0,
    #[serde(rename = "GrindCurrency")]
    #[strum(serialize = "GrindCurrency")]
    Grind = 1,
    #[serde(rename = "MissionCurrency")]
    #[strum(serialize = "MissionCurrency")]
    Mission = 2,
}

#[derive(Debug, Error)]
#[error("unknown currency repr")]
pub struct UnknownCurrencyRepr;

impl TryFrom<u8> for CurrencyType {
    type Error = UnknownCurrencyRepr;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_repr(value).ok_or(UnknownCurrencyRepr)
    }
}

/// Unknown currency error
#[derive(Debug, Error)]
#[error("unknown currency type")]
pub struct UnknownCurrency;

impl FromStr for CurrencyType {
    type Err = UnknownCurrency;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "MTXCurrency" => Self::Mtx,
            "GrindCurrency" => Self::Grind,
            "MissionCurrency" => Self::Mission,
            _ => return Err(UnknownCurrency),
        })
    }
}
