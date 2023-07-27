use std::{collections::HashMap, fs::File, io::BufWriter};

use csv::ReaderBuilder;
use log::debug;

const I18N_TRANSLATIONS: &[u8] = include_bytes!("../../resources/data/i18n.csv");

pub struct I18nService {
    pub map: HashMap<u32, String>,
}

impl I18nService {
    pub fn new() -> Self {
        let mut map = ReaderBuilder::new()
            .from_reader(I18N_TRANSLATIONS)
            .into_records()
            .flatten()
            .filter_map(|record| {
                let key = record.get(0)?.parse().ok()?;
                let value = record.get(1)?;

                Some((key, value.to_string()))
            })
            .collect();

        Self { map }
    }

    pub fn get(&self, key: u32) -> Option<&str> {
        match self.map.get(&key) {
            Some(value) => Some(value),
            None => None,
        }
    }
}
