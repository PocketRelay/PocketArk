use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    database::entity::{currency::CurrencyType, Currency, InventoryItem},
    services::{
        activity::ActivityResult,
        items::ItemDefinition,
        store::{StoreArticleName, StoreCatalog},
    },
};

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
