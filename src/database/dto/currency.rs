use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

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
