//! # Translations
//!
//! Provides translation mappings between game translation ids and actual
//! translated text
//!
//! Translation mappings are stored in the csv file at `src/resources/data/i18n.csv`

use std::{fmt::Debug, sync::OnceLock};

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
    pub fn lookup(&self, key: &u32) -> Option<&str> {
        self.map.get(key).map(|value| value.as_ref())
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

impl<T> Localized for &mut [T]
where
    T: Localized,
{
    fn localize(&mut self, i18n: &I18n) {
        self.iter_mut().for_each(|value| value.localize(i18n))
    }
}

/// Serializable structure for including the i18n name and localized
/// name in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nName {
    /// I18n lookup key
    pub i18n_name: I18nKey,
    /// Localized translated name
    pub loc_name: Option<ImStr>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum I18nKey {
    /// Valid i18n lookup key
    Lookup(#[serde_as(as = "serde_with::DisplayFromStr")] u32),
    /// Some raw string value that can't be used to lookup with
    ///
    /// Thanks to EA and some of their translations not actually
    /// containing the lookup key instead containg something
    /// like: "FREE_1100_APEX_POINTS_ON_ADD"
    Raw(ImStr),
}

impl I18nName {
    pub const fn new(i18n_name: u32) -> Self {
        Self {
            i18n_name: I18nKey::Lookup(i18n_name),
            loc_name: None,
        }
    }
}

impl Debug for I18nName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_name: Option<&str> = self
            .loc_name
            .as_ref()
            .map(|value| value.as_ref())
            // Attempt to load the translation for debug logging
            .or_else(|| {
                if let I18nKey::Lookup(key) = &self.i18n_name {
                    let i18n = I18n::get();
                    i18n.lookup(key)
                } else {
                    None
                }
            });

        f.debug_struct("I18nName")
            .field("i18n_name", &self.i18n_name)
            .field("loc_name", &loc_name)
            .finish()
    }
}

impl Localized for I18nName {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_name.is_some() {
            return;
        }

        let i18n_name = match &self.i18n_name {
            I18nKey::Lookup(value) => value,
            _ => return,
        };

        self.loc_name = i18n.lookup(i18n_name).map(Box::from);
    }
}

/// Serializable structure for including the i18n title and localized
/// title in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nTitle {
    /// I18n lookup key
    pub i18n_title: I18nKey,
    /// Localized translated title
    pub loc_title: Option<ImStr>,
}

impl I18nTitle {
    pub const fn new(i18n_title: u32) -> Self {
        Self {
            i18n_title: I18nKey::Lookup(i18n_title),
            loc_title: None,
        }
    }
}

impl Debug for I18nTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_title: Option<&str> = self
            .loc_title
            .as_ref()
            .map(|value| value.as_ref())
            // Attempt to load the translation for debug logging
            .or_else(|| {
                if let I18nKey::Lookup(key) = &self.i18n_title {
                    let i18n = I18n::get();
                    i18n.lookup(key)
                } else {
                    None
                }
            });

        f.debug_struct("I18nTitle")
            .field("i18n_title", &self.i18n_title)
            .field("loc_title", &loc_title)
            .finish()
    }
}

impl Localized for I18nTitle {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_title.is_some() {
            return;
        }

        let i18n_title = match &self.i18n_title {
            I18nKey::Lookup(value) => value,
            _ => return,
        };

        self.loc_title = i18n.lookup(i18n_title).map(Box::from);
    }
}

/// Serializable structure for including the i18n description
/// and localized description in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nDescription {
    /// I18n lookup key
    pub i18n_description: I18nKey,
    /// Localized translated description
    pub loc_description: Option<ImStr>,
}

impl I18nDescription {
    pub const fn new(i18n_description: u32) -> Self {
        Self {
            i18n_description: I18nKey::Lookup(i18n_description),
            loc_description: None,
        }
    }
}

impl Debug for I18nDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_description: Option<&str> = self
            .loc_description
            .as_ref()
            .map(|value| value.as_ref())
            // Attempt to load the translation for debug logging
            .or_else(|| {
                if let I18nKey::Lookup(key) = &self.i18n_description {
                    let i18n = I18n::get();
                    i18n.lookup(key)
                } else {
                    None
                }
            });

        f.debug_struct("I18nDescription")
            .field("i18n_description", &self.i18n_description)
            .field("loc_description", &loc_description)
            .finish()
    }
}

impl Localized for I18nDescription {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_description.is_some() {
            return;
        }

        let i18n_description = match &self.i18n_description {
            I18nKey::Lookup(value) => value,
            _ => return,
        };

        self.loc_description = i18n.lookup(i18n_description).map(Box::from);
    }
}

/// Serializable structure for including the i18n description
/// and localized description in JSON (Shorthand because thanks EA)
///
/// TODO: Maybe use serde_as to alias this from [I18nDescription]
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nDesc {
    /// I18n lookup key
    pub i18n_desc: I18nKey,
    /// Localized translated description
    pub loc_desc: Option<ImStr>,
}

impl I18nDesc {
    pub const fn new(i18n_desc: u32) -> Self {
        Self {
            i18n_desc: I18nKey::Lookup(i18n_desc),
            loc_desc: None,
        }
    }
}

impl Debug for I18nDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_desc: Option<&str> = self
            .loc_desc
            .as_ref()
            .map(|value| value.as_ref())
            // Attempt to load the translation for debug logging
            .or_else(|| {
                if let I18nKey::Lookup(key) = &self.i18n_desc {
                    let i18n = I18n::get();
                    i18n.lookup(key)
                } else {
                    None
                }
            });

        f.debug_struct("I18nDesc")
            .field("i18n_desc", &self.i18n_desc)
            .field("loc_desc", &loc_desc)
            .finish()
    }
}

impl Localized for I18nDesc {
    fn localize(&mut self, i18n: &I18n) {
        // Already localized
        if self.loc_desc.is_some() {
            return;
        }

        let i18n_desc = match &self.i18n_desc {
            I18nKey::Lookup(value) => value,
            _ => return,
        };

        self.loc_desc = i18n.lookup(i18n_desc).map(Box::from);
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
