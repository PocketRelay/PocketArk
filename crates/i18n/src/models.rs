use crate::{I18n, I18nKey, Localized};
use pocket_ark_core::ImStr;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::fmt::Debug;

/// Serializable structure for including the i18n name and localized
/// name in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nName {
    /// I18n lookup key
    pub i18n_name: I18nKey,
    /// Localized translated name
    pub loc_name: Option<ImStr>,
}

impl I18nName {
    pub const fn new(i18n_name: u32) -> Self {
        Self {
            i18n_name: I18nKey::Lookup(i18n_name),
            loc_name: None,
        }
    }

    pub fn raw(i18n_name: impl Into<ImStr>) -> Self {
        Self {
            i18n_name: I18nKey::Raw(i18n_name.into()),
            loc_name: None,
        }
    }
}

impl Debug for I18nName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_name: Option<&ImStr> = self
            .loc_name
            .as_ref()
            // Attempt to load the translation for debug logging
            .or_else(|| {
                let i18n = I18n::get();
                i18n.by_key(&self.i18n_name)
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

        self.loc_name = i18n.by_key(&self.i18n_name).cloned();
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
    #[allow(unused)]
    pub const fn new(i18n_title: u32) -> Self {
        Self {
            i18n_title: I18nKey::Lookup(i18n_title),
            loc_title: None,
        }
    }

    pub fn raw(i18n_title: impl Into<ImStr>) -> Self {
        Self {
            i18n_title: I18nKey::Raw(i18n_title.into()),
            loc_title: None,
        }
    }
}

impl Debug for I18nTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_title: Option<&ImStr> = self
            .loc_title
            .as_ref()
            // Attempt to load the translation for debug logging
            .or_else(|| {
                let i18n = I18n::get();
                i18n.by_key(&self.i18n_title)
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

        self.loc_title = i18n.by_key(&self.i18n_title).cloned();
    }
}

/// Serializable structure for including the i18n description
/// and localized description in JSON
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn raw(i18n_description: impl Into<ImStr>) -> Self {
        Self {
            i18n_description: I18nKey::Raw(i18n_description.into()),
            loc_description: None,
        }
    }
}

impl Debug for I18nDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loc_description: Option<&ImStr> = self
            .loc_description
            .as_ref()
            // Attempt to load the translation for debug logging
            .or_else(|| {
                let i18n = I18n::get();
                i18n.by_key(&self.i18n_description)
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

        self.loc_description = i18n.by_key(&self.i18n_description).cloned();
    }
}

/// Serializable structure for including the i18n description
/// and localized description in JSON (Shorthand because thanks EA)
///
/// TODO: Maybe use serde_as to alias this from [I18nDescription]
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        let loc_desc: Option<&ImStr> = self
            .loc_desc
            .as_ref()
            // Attempt to load the translation for debug logging
            .or_else(|| {
                let i18n = I18n::get();
                i18n.by_key(&self.i18n_desc)
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

        self.loc_desc = i18n.by_key(&self.i18n_desc).cloned();
    }
}
