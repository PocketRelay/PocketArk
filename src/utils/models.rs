use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Localized naming variables
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleNameWithDesc {
    // translation codes
    pub i18n_name: String,
    pub i18n_description: Option<String>,

    // translated
    pub loc_name: Option<String>,
    pub loc_description: Option<String>,
}

impl LocaleNameWithDesc {
    /// Returns the localized name if present otherwise
    /// returns the i18n translation code
    pub fn name(&self) -> &str {
        match self.loc_name.as_ref() {
            Some(value) => value,
            None => &self.i18n_name,
        }
    }

    /// Returns the localized description if present otherwise
    /// returns the i18n translation code
    pub fn description(&self) -> Option<&String> {
        self.loc_description
            .as_ref()
            .or(self.i18n_description.as_ref())
    }
}

/// Localized naming variables
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleName {
    // translation codes
    pub i18n_name: String,
    // translated
    pub loc_name: Option<String>,
}

impl LocaleName {
    /// Returns the localized name if present otherwise
    /// returns the i18n translation code
    pub fn name(&self) -> &str {
        match self.loc_name.as_ref() {
            Some(value) => value,
            None => &self.i18n_name,
        }
    }
}
