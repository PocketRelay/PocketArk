use std::process::exit;

use chrono::{DateTime, Utc};
use log::error;
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{database::entity::currency::CurrencyType, utils::models::LocaleNameWithDesc};

/// Definition file for the contents of the in-game store
const STORE_CATALOG_DEFINITION: &str = include_str!("../../resources/data/storeCatalog.json");

pub struct StoreService {
    pub catalog: StoreCatalog,
}

impl StoreService {
    pub fn new() -> Self {
        let catalog: StoreCatalog = match serde_json::from_str(STORE_CATALOG_DEFINITION) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load store definitions: {}", err);
                exit(1);
            }
        };

        Self { catalog }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCatalog {
    pub catalog_id: String,
    pub name: String,

    pub custom_attributes: Map<String, Value>,
    pub categories: Vec<String>,
    pub articles: Vec<StoreArticle>,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

pub type StoreArticleName = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreArticle {
    pub catalog_name: String,

    pub categories: Vec<String>,
    pub custom_attributes: Map<String, Value>,
    pub nucleus_entitlement_filter: Map<String, Value>,
    pub prices: Vec<StorePrice>,
    pub limits: Vec<StoreLimit>,
    pub item_name: Uuid,
    pub name: StoreArticleName,
    pub auto_claim: bool,
    pub available_grace_in_seconds: u32,
    pub limited_availability: bool,
    pub available_duration: StoreDuration,
    pub visible_duration: StoreDuration,
    pub seen: bool,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreLimit {
    pub scope: String,
    pub maximum: u32,
    pub quantity_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorePrice {
    pub currency: CurrencyType,
    pub original_price: u32,
    pub final_price: u32,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreDuration {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTimeUtc>,
}
