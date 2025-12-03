use super::HttpError;
use crate::{
    database::entity::{currency::CurrencyType, Currency, InventoryItem},
    definitions::{
        items::ItemDefinition,
        store_catalogs::{StoreArticleName, StoreCatalog},
    },
    services::activity::ActivityResult,
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
}

impl HttpError for StoreError {
    fn status(&self) -> StatusCode {
        match self {
            StoreError::UnknownArticle => StatusCode::NOT_FOUND,
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
pub struct ClaimUnclaimedResponse {
    pub claim_results: Vec<Value>,
    pub results_complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct UpdateSeenArticles {
    pub article_names: Vec<Uuid>,
}
