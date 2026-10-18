use crate::currency::CurrencyType;
use pocket_ark_core::custom_attributes::CustomAttributes;
use pocket_ark_i18n::{I18n, I18nDescription, I18nName, Localized};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

pub struct StrikeTeamEquipmentList {
    pub equipment: Vec<StrikeTeamEquipment>,
}

impl StrikeTeamEquipmentList {
    pub fn from_bytes(value: &[u8]) -> serde_json::Result<Self> {
        let equipment: Vec<StrikeTeamEquipment> = serde_json::from_slice(value)?;
        Ok(Self { equipment })
    }
}

/// Type alias for a [String] representing the name of a [StrikeTeamEquipment]
pub type StrikeTeamEquipmentName = String;

/// Equipment that a strike team can purchase
///
/// For reference: https://masseffectandromeda.fandom.com/wiki/Strike_team#Equipment
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamEquipment {
    /// Unique name for the equipment
    pub name: StrikeTeamEquipmentName,

    /// Strike team level required to purchase the equipment
    pub level_required: u32,

    /// Effectiveness boost given by the equipment
    pub effectiveness: u32,

    /// Optional collection of tags that are affected by this
    /// equipment, not present if effect is always applied
    pub tags: Option<Vec<String>>,

    /// Cost of the equipment for different currency types
    pub cost_by_currency: HashMap<CurrencyType, u32>,

    /// Additional custom attributes
    pub custom_attributes: CustomAttributes,

    /// Localized equipment name
    #[serde(flatten)]
    pub i18n_name: I18nName,
    /// Localized equipment description
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
}

impl Localized for StrikeTeamEquipment {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        self.i18n_description.localize(i18n);
    }
}
