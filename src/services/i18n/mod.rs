//! Service that provides translations, translations are loaded from a .csv file
//! at src/resources/data/i18n.csv

use std::sync::OnceLock;

use crate::utils::hashing::IntHashMap;
use csv::ReaderBuilder;
use log::debug;

/// Translations (103400)
const I18N_TRANSLATIONS: &[u8] = include_bytes!("../../resources/data/i18n.csv");

/// Translation service
pub struct I18n {
    /// Mapping between translation keys and the actual translation value
    map: IntHashMap<u32, String>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<I18n> = OnceLock::new();

impl I18n {
    /// Gets a static reference to the global [I18nService] collection
    pub fn get() -> &'static I18n {
        STORE.get_or_init(|| Self::new())
    }

    /// Creates a new i18n service
    pub fn new() -> Self {
        let map: IntHashMap<u32, String> = ReaderBuilder::new()
            .from_reader(I18N_TRANSLATIONS)
            .into_records()
            .flatten()
            .filter_map(|record| {
                let key = record.get(0)?.parse().ok()?;
                let value = record.get(1)?;

                Some((key, value.to_string()))
            })
            .collect();

        debug!("Loaded {} translation(s)", map.len());

        Self { map }
    }

    /// Attempts to find a specific translation from its translation key
    pub fn lookup(&self, key: u32) -> Option<&str> {
        match self.map.get(&key) {
            Some(value) => Some(value),
            None => None,
        }
    }
}
