use serde::{Deserialize, Serialize};
use uuid::{Uuid, uuid};

use crate::definitions::{
    i18n::{I18n, I18nDesc, I18nName, Localized},
    shared::CustomAttributes,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionType {
    /// The unique ID name for the type
    pub name: Uuid,
    /// Descriptor for the mission
    pub descriptor: MissionTypeDescriptor,
    /// Whether the mission gives currency rewards
    pub give_currency: bool,
    /// Whether the mission gives XP
    pub give_xp: bool,
}

impl Default for MissionType {
    fn default() -> Self {
        Self {
            name: uuid!("1cedd0c2-652b-d879-d8c9-0ff8b1b0bf9c"),
            descriptor: Default::default(),
            give_currency: true,
            give_xp: true,
        }
    }
}

impl Localized for MissionType {
    fn localize(&mut self, i18n: &I18n) {
        self.descriptor.localize(i18n);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionTypeDescriptor {
    pub name: Uuid,
    #[serde(flatten)]
    pub i18n_name: I18nName,

    #[serde(flatten)]
    pub i18n_desc: Option<I18nDesc>,

    pub custom_attributes: CustomAttributes,
}

impl Default for MissionTypeDescriptor {
    fn default() -> Self {
        Self {
            name: uuid!("39b9880a-ce11-4be3-a3e7-728763b48614"),
            i18n_name: I18nName::new(12028 /* "Normal" */),
            i18n_desc: None,
            custom_attributes: Default::default(),
        }
    }
}

impl Localized for MissionTypeDescriptor {
    fn localize(&mut self, i18n: &I18n) {
        self.i18n_name.localize(i18n);
        if let Some(i18n_desc) = &mut self.i18n_desc {
            i18n_desc.localize(i18n);
        }
    }
}
