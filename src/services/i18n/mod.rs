//! Service that provides translations, translations are loaded from a .csv file
//! at src/resources/data/i18n.csv

use crate::utils::hashing::IntHashMap;
use csv::ReaderBuilder;
use log::debug;

const I18N_TRANSLATIONS: &[u8] = include_bytes!("../../resources/data/i18n.csv");

/// Translation service
pub struct I18nService {
    /// Mapping between translation keys and the actual translation value
    map: IntHashMap<u32, String>,
}

impl I18nService {
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
    pub fn get(&self, key: u32) -> Option<&str> {
        match self.map.get(&key) {
            Some(value) => Some(value),
            None => None,
        }
    }
}
