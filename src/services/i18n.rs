//! # Translations
//!
//! Provides translation mappings between game translation ids and actual
//! translated text
//!
//! Translation mappings are stored in the csv file at `src/resources/data/i18n.csv`

use std::sync::OnceLock;

use crate::utils::{
    hashing::{int_hash_map, IntHashMap},
    ImStr,
};
use anyhow::Context;
use csv::ReaderBuilder;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// Translations (103400)
const I18N_TRANSLATIONS: &[u8] = include_bytes!("../resources/data/i18n.csv");

/// Translation service
pub struct I18n {
    /// Mapping between translation keys and the actual translation value
    map: IntHashMap<u32, ImStr>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<I18n> = OnceLock::new();

impl I18n {
    /// Gets a static reference to the global [I18nService] collection
    pub fn get() -> &'static I18n {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    /// Creates a new [I18n] collection, loading the translations
    /// from the embedded file [I18N_TRANSLATIONS]
    fn load() -> anyhow::Result<Self> {
        let mut map = int_hash_map();

        let records = ReaderBuilder::new()
            .from_reader(I18N_TRANSLATIONS)
            .into_records();

        for record in records {
            let record = record.context("Failed to parse translation record")?;
            let key = record
                .get(0)
                .context("Translation record missing key")?
                .parse::<u32>()
                .context("Translation key was invalid")?;
            let value = record.get(1).context("Translation record missing value")?;
            map.insert(key, Box::from(value));
        }

        debug!("Loaded {} translation(s)", map.len());

        Ok(Self { map })
    }

    /// Attempts to find a specific translation from its translation key
    pub fn lookup(&self, key: u32) -> Option<&str> {
        self.map.get(&key).map(|value| value.as_ref())
    }
}

/// Trait implemented by structures that can
/// be localized
pub trait Localized: Sized {
    /// Localizes the structure using the provided `i18n`
    /// definitions
    fn localize(&mut self, i18n: &I18n);
}

/// Serializable structure for including the i18n name and localized
/// name in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nName {
    /// I18n lookup key
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub i18n_name: u32,
    /// Localized translated name
    pub loc_name: Option<ImStr>,
}

impl Localized for I18nName {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_name.is_some() {
            return;
        }

        self.loc_name = i18n.lookup(self.i18n_name).map(Box::from);
    }
}

/// Serializable structure for including the i18n title and localized
/// title in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nTitle {
    /// I18n lookup key
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub i18n_title: u32,
    /// Localized translated title
    pub loc_title: Option<ImStr>,
}

impl Localized for I18nTitle {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_title.is_some() {
            return;
        }

        self.loc_title = i18n.lookup(self.i18n_title).map(Box::from);
    }
}

/// Serializable structure for including the i18n description
/// and localized description in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nDescription {
    /// I18n lookup key
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub i18n_description: u32,
    /// Localized translated description
    pub loc_description: Option<ImStr>,
}

impl Localized for I18nDescription {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_description.is_some() {
            return;
        }

        self.loc_description = i18n.lookup(self.i18n_description).map(Box::from);
    }
}

#[cfg(test)]
mod test {
    use super::I18n;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = I18n::load().unwrap();
    }
}
