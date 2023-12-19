use std::process::exit;

use chrono::{DateTime, Utc};
use log::error;
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    database::entity::currency::CurrencyType,
    utils::models::{DateDuration, LocaleNameWithDesc},
};

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

/// Type alias for a string representing a store catalog name
pub type StoreCatalogName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCatalog {
    pub catalog_id: StoreCatalogName,
    pub name: String,

    pub custom_attributes: Map<String, Value>,
    pub categories: Vec<String>,
    pub articles: Vec<StoreArticle>,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

impl StoreCatalog {
    pub fn get_article(&self, article_name: &StoreArticleName) -> Option<&StoreArticle> {
        self.articles
            .iter()
            .find(|article| article.name.eq(article_name))
    }
}

pub type StoreArticleName = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreArticle {
    pub catalog_name: StoreCatalogName,

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
    pub available_duration: DateDuration,
    pub visible_duration: DateDuration,
    pub seen: bool,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

impl StoreArticle {
    /// Retrieves the [StorePrice] for this article for a specific
    /// currency type
    pub fn price_by_currency(&self, currency: CurrencyType) -> Option<&StorePrice> {
        self.prices
            .iter()
            // Find a price with the provided `currency`
            .find(|price| price.currency == currency)
    }
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
