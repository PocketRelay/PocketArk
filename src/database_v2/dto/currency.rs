use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::EnumIter;

use crate::database_v2::dto::users::UserId;

#[derive(Debug, FromRow)]
pub struct CurrencyDto {
    pub user_id: UserId,
    pub ty: CurrencyType,
    pub balance: u32,
}

#[derive(Debug, Clone, EnumIter, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[repr(u8)]
pub enum CurrencyType {
    #[serde(rename = "MTXCurrency")]
    Mtx = 0,
    #[serde(rename = "GrindCurrency")]
    Grind = 1,
    #[serde(rename = "MissionCurrency")]
    Mission = 2,
}

#[derive(Debug)]
pub struct CurrencyUpdateDto {
    pub ty: CurrencyType,
    pub balance: i32,
}
