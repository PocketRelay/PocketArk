use serde::{Deserialize, Serialize};

use super::inventory::{ActivityResult, InventoryItem, ItemDefinition};

#[derive(Debug, Deserialize, Serialize)]
pub struct Currency {
    pub name: String,
    pub balance: u32,
}

#[derive(Serialize)]
pub struct UserCurrenciesResponse {
    pub list: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObtainStoreItemRequest {
    pub currency: String,
    pub article_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObtainStoreItemResponse {
    pub generated_activity_result: ActivityResult,
    pub items: Vec<InventoryItem>,
    pub definitions: Vec<&'static ItemDefinition>,
}
