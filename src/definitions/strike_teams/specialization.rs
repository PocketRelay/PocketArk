use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::definitions::{
    i18n::{I18n, I18nDescription, I18nName, Localized},
    shared::CustomAttributes,
};

const STRIKE_TEAM_SPECIALIZATION_DEFINITIONS: &str =
    include_str!("../../resources/data/strikeTeamSpecialization.json");

pub struct StrikeTeamSpecializations {
    pub specializations: Vec<StrikeTeamSpecialization>,
}

impl StrikeTeamSpecializations {
    pub fn load() -> serde_json::Result<Self> {
        Self::from_str(STRIKE_TEAM_SPECIALIZATION_DEFINITIONS)
    }
}

impl FromStr for StrikeTeamSpecializations {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let specializations: Vec<StrikeTeamSpecialization> = serde_json::from_str(s)?;
        Ok(Self { specializations })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamSpecialization {
    /// Name of the specialization
    pub name: String,
    /// The tag that the specialization affects
    pub tag: String,
    /// The effectiveness of the specialization
    pub effectiveness: u32,
    /// Additional custom attributes (Appears unused in official config)
    pub custom_attributes: CustomAttributes,

    /// Localized specialization name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized specialization description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

impl Localized for StrikeTeamSpecialization {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        self.i18n_description.localize(i18n);
    }
}
