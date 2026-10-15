//! # Translations
//!
//! Provides translation mappings between game translation ids and actual
//! translated text
//!
//! Translation mappings are stored in the csv file at `translations/en.csv`

use anyhow::Context;
use csv::ReaderBuilder;
use pocket_ark_core::{ImStr, IntHashMap, int_hash_map};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{fmt::Debug, sync::OnceLock};
use tracing::debug;

pub mod models;

pub use models::{I18nDesc, I18nDescription, I18nName, I18nTitle};

/// Translations (103400)
const I18N_TRANSLATIONS: &[u8] = include_bytes!("../translations/en.csv");

/// Type alias for a lookup key in the i18n map
pub type LookupKey = u32;

/// Translation key, requires handling for raw string types
///
/// Thanks to EA and some of their translations not actually
/// containing the lookup key instead containing something
/// like: "FREE_1100_APEX_POINTS_ON_ADD"
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum I18nKey {
    /// Valid i18n lookup key
    Lookup(#[serde_as(as = "serde_with::DisplayFromStr")] LookupKey),
    /// Some raw string value that can't be used to lookup with
    Raw(ImStr),
}

/// Translation definitions
pub struct I18n {
    /// Mapping between translation keys and the actual translation value
    map: IntHashMap<LookupKey, ImStr>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<I18n> = OnceLock::new();

impl I18n {
    /// Gets a static reference to the global [I18n] collection
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
    pub fn by_key(&self, key: &I18nKey) -> Option<&ImStr> {
        match key {
            I18nKey::Lookup(value) => self.map.get(value),
            I18nKey::Raw(_) => None,
        }
    }
}

/// Trait implemented by structures that can
/// be localized
pub trait Localized: Sized {
    /// Localizes the structure using the provided `i18n`
    /// definitions
    fn localize(&mut self, i18n: &I18n);
}

impl<T> Localized for Vec<T>
where
    T: Localized,
{
    fn localize(&mut self, i18n: &I18n) {
        self.iter_mut().for_each(|value| value.localize(i18n))
    }
}

impl<T> Localized for Option<T>
where
    T: Localized,
{
    fn localize(&mut self, i18n: &I18n) {
        if let Some(value) = self.as_mut() {
            value.localize(i18n);
        }
    }
}

impl<T> Localized for &mut [T]
where
    T: Localized,
{
    fn localize(&mut self, i18n: &I18n) {
        self.iter_mut().for_each(|value| value.localize(i18n))
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
