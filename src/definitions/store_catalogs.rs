use super::{
    i18n::{I18nDescription, I18nName},
    items::ItemName,
};
use crate::{database::entity::currency::CurrencyType, utils::models::DateDuration};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::OnceLock;
use uuid::Uuid;

/// Definition file for the contents of the in-game store
const STORE_CATALOG_DEFINITION: &str = include_str!("../resources/data/storeCatalog.json");

pub struct StoreCatalogs {
    pub catalog: StoreCatalog,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<StoreCatalogs> = OnceLock::new();

impl StoreCatalogs {
    /// Gets a static reference to the global [StoreCatalogs] collection
    pub fn get() -> &'static StoreCatalogs {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let catalog: StoreCatalog = serde_json::from_str(STORE_CATALOG_DEFINITION)
            .context("Failed to load store catalog definitions")?;

        Ok(Self { catalog })
    }
}

/// Type alias for a string representing a store catalog name
pub type StoreCatalogName = String;

/// Catalog aka collection of [StoreArticle]s, in this case the game only
/// has a "Standard" catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCatalog {
    /// The ID of the catalog (Same as `name`)
    pub catalog_id: StoreCatalogName,
    /// The name of the catalog
    pub name: String,
    /// Custom attributes associated with the catalog (Haven't seen this with any values)
    pub custom_attributes: Map<String, Value>,
    /// Categories this catalog falls under (Unknown value meanings, needs further attention)
    pub categories: Vec<String>,
    /// Articles present in the catalog
    pub articles: Vec<StoreArticle>,

    /// Localized catalog name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized catalog description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

impl StoreCatalog {
    pub fn get_article(&self, article_name: &StoreArticleName) -> Option<&StoreArticle> {
        self.articles
            .iter()
            .find(|article| article.name.eq(article_name))
    }
}

/// Type alias for a [Uuid] representing the name of a [StoreArticle]
pub type StoreArticleName = Uuid;

/// Represents an item within a [StoreCatalog] that can be
/// purchased
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreArticle {
    /// Unique ID for this article item (Identifies which one to purchase)
    pub name: StoreArticleName,
    /// The name of the [StoreCatalog] this article belongs to
    pub catalog_name: StoreCatalogName,

    /// Categories this article falls under (Unknown value meanings, needs further attention)
    pub categories: Vec<String>,
    /// Attributes associated with the article, used for things such as the
    /// image shown for the article
    pub custom_attributes: Map<String, Value>,
    /// Filtering based on user entitlements. Haven't seen any usage of
    /// this so structure is unknown, likely similar to attribute filters
    pub nucleus_entitlement_filter: Map<String, Value>,
    /// Lists the price of the article across the different currencies
    /// that can be used  
    pub prices: Vec<StorePrice>,
    /// Limits on the amount of this article that can be purchased. This is
    /// a vec because the limits can be applied to different scopes
    ///
    /// TODO: Per-article purchasing limits need to be created and this
    /// needs to be store in the database to track [StoreLimit::quantity_remaining]
    pub limits: Vec<StoreLimit>,
    /// Name of the item definition this will grant upon purchase
    pub item_name: ItemName,
    /// Whether to automatically claim the article item
    /// (Have only seen this set to false, so actual usage is unknown)
    pub auto_claim: bool,
    /// Unknown usage. TODO: Investigate
    pub available_grace_in_seconds: u32,
    /// Whether this article is a limited time item
    pub limited_availability: bool,
    /// An optional duration this article should only be available for
    pub available_duration: DateDuration,
    /// An optional duration this article should only be visible for
    pub visible_duration: DateDuration,
    /// Seen state, currently not implemented. Will likely override this
    /// later to always be true.
    ///
    /// TODO: If per-item limits are added this can probabbly be included in that
    /// database table for simplicity
    pub seen: bool,

    /// Localized article name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized article description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
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

/// Limit for a [StoreArticle]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreLimit {
    /// The scope of the limit (Seen: "USER")
    pub scope: String,
    /// The maximum number of times the article can be purchased
    pub maximum: u32,
    /// The remaining number of items that can be purchased.
    ///
    /// TODO: This should probabbly be store in the database..?
    pub quantity_remaining: u32,
}

/// Price of a [StoreArticle] for a specific currency
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorePrice {
    /// The type of currency this price is for
    pub currency: CurrencyType,
    /// The original price of the item if the item is on discount
    pub original_price: u32,
    /// The final cost price of the item (The actual price)
    pub final_price: u32,
}
#[cfg(test)]
mod test {
    use super::StoreCatalogs;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = StoreCatalogs::load().unwrap();
    }
}
