use pocket_ark_core::custom_attributes::CustomAttributes;
use pocket_ark_i18n::{I18n, I18nDescription, I18nName, Localized};
use serde::{Deserialize, Serialize};

pub struct StrikeTeamSpecializations {
    pub specializations: Vec<StrikeTeamSpecialization>,
}

impl StrikeTeamSpecializations {
    pub fn from_bytes(value: &[u8]) -> serde_json::Result<Self> {
        let specializations: Vec<StrikeTeamSpecialization> = serde_json::from_slice(value)?;
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
