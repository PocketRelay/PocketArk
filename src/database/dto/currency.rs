use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumIter};
use thiserror::Error;

use crate::database::dto::users::UserId;

#[derive(Debug, Clone, FromRow, Serialize, PartialEq, Eq, Deserialize)]
pub struct CurrencyDto {
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "name")]
    pub ty: CurrencyType,
    pub balance: u32,
}

#[derive(
    Debug, Clone, EnumIter, Copy, PartialEq, Eq, Hash, Display, Serialize, Deserialize, sqlx::Type,
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

#[derive(Debug)]
pub struct CurrencyUpdateDto {
    pub ty: CurrencyType,
    pub balance: i32,
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
