//! # Translations
//!
//! Provides translation mappings between game translation ids and actual
//! translated text
//!
//! Translation mappings are stored in the csv file at `src/resources/data/i18n.csv`

use std::sync::OnceLock;

use crate::utils::hashing::{int_hash_map, IntHashMap};
use anyhow::Context;
use csv::ReaderBuilder;
use log::debug;

/// Translations (103400)
const I18N_TRANSLATIONS: &[u8] = include_bytes!("../resources/data/i18n.csv");

/// Translation service
pub struct I18n {
    /// Mapping between translation keys and the actual translation value
    map: IntHashMap<u32, Box<str>>,
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

#[cfg(test)]
mod test {
    use super::I18n;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = I18n::load().unwrap();
    }
}
