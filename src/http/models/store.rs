use super::HttpError;
use crate::{
    database::entity::{currency::CurrencyType, Currency, InventoryItem},
    services::{
        activity::ActivityResult,
        catalogs::{StoreArticleName, StoreCatalog},
        items::ItemDefinition,
    },
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    /// Couldn't find the article requested
    #[error("Unknown article")]
    UnknownArticle,
    /// Article cannot be purchased with the requested currency
    #[error("Invalid currency")]
    InvalidCurrency,

    /// User doesn't have enough currency to purchase the item
    #[error("Currency balance cannot be less than 0.")]
    InsufficientCurrency,
}

impl HttpError for StoreError {
    fn status(&self) -> StatusCode {
        match self {
            StoreError::UnknownArticle => StatusCode::NOT_FOUND,
            StoreError::InvalidCurrency | StoreError::InsufficientCurrency => StatusCode::CONFLICT,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCatalogResponse {
    pub list: Vec<&'static StoreCatalog>,
}

#[derive(Serialize)]
pub struct UserCurrenciesResponse {
    pub list: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObtainStoreItemRequest {
    pub currency: CurrencyType,
    pub article_name: StoreArticleName,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObtainStoreItemResponse {
    pub generated_activity_result: ActivityResult,
    pub items: Vec<InventoryItem>,
    pub definitions: Vec<&'static ItemDefinition>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimUncalimedResponse {
    pub claim_results: Vec<Value>,
    pub results_complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeenArticles {
    pub article_names: Vec<Uuid>,
}
