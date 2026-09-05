use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use thiserror::Error;

use crate::{database::dto::users::UserId, definitions::currency::CurrencyType};

#[derive(Debug, Clone, FromRow, Serialize, PartialEq, Eq, Deserialize)]
pub struct CurrencyDto {
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "name")]
    #[sqlx(try_from = "u8")]
    pub ty: CurrencyType,
    pub balance: u32,
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
