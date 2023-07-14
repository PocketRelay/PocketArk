use std::{collections::HashMap, fmt};

use crate::database::entity::{Character, SharedData};

use chrono::{DateTime, Utc};
use sea_orm::FromJsonQueryResult;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharactersResponse {
    pub list: Vec<Character>,
    #[serde(flatten)]
    pub shared_data: SharedData,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomizationRequest {
    pub customization: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSkillTreesRequest {
    pub skill_trees: Vec<SkillTreeEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterResponse {
    #[serde(flatten)]
    pub character: Character,
    #[serde(flatten)]
    pub shared_data: SharedData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Xp {
    pub current: u32,
    pub last: u32,
    pub next: u32,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillTreeEntry {
    pub name: Uuid,
    pub tree: Vec<SkillTreeTier>,
    pub timestamp: Option<DateTime<Utc>>,
    pub obsolete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillTreeTier {
    pub tier: u32,
    pub skills: HashMap<String, u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterEquipment {
    pub slot: String,
    pub name: MaybeUuid,
    pub attachments: Vec<Uuid>,
}

use serde_with::DisplayFromStr;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationEntry {
    #[serde_as(as = "DisplayFromStr")]
    pub value_x: f32,
    #[serde_as(as = "DisplayFromStr")]
    pub value_y: f32,
    #[serde_as(as = "DisplayFromStr")]
    pub value_z: f32,
    #[serde_as(as = "DisplayFromStr")]
    pub value_w: f32,
    #[serde(rename = "type")]
    #[serde_as(as = "DisplayFromStr")]
    pub ty: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub param_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterLevelTables {
    pub list: Vec<LevelTable>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTable {
    pub table: Vec<LevelTableEntry>,
    pub name: Uuid,
    pub i18n_name: String,
    pub i18n_description: String,
    pub loc_name: Option<String>,
    pub loc_description: Option<String>,
    pub custom_attributes: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTableEntry {
    pub level: u32,
    pub xp: u32,
    pub rewards: HashMap<String, f64>,
    pub custom_attributes: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterEquipmentList {
    pub list: Vec<CharacterEquipment>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MaybeUuid(pub Option<Uuid>);

impl Serialize for MaybeUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = &self.0 {
            value.serialize(serializer)
        } else {
            serializer.serialize_str("")
        }
    }
}

impl<'de> Deserialize<'de> for MaybeUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmptyOrUuid;

        impl<'de> Visitor<'de> for EmptyOrUuid {
            type Value = MaybeUuid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("empty string or UUID")
            }

            fn visit_str<E>(self, value: &str) -> Result<MaybeUuid, E>
            where
                E: serde::de::Error,
            {
                if value.is_empty() {
                    Ok(MaybeUuid(None))
                } else {
                    Uuid::parse_str(value)
                        .map_err(|e| {
                            serde::de::Error::custom(format_args!("UUID parsing failed: {}", e))
                        })
                        .map(|value| MaybeUuid(Some(value)))
                }
            }
        }

        deserializer.deserialize_str(EmptyOrUuid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDefinition {
    pub i18n_name: String,
    pub i18n_description: String,
    pub custom_attributes: Map<String, Value>,
    pub tiers: Vec<SkillDefinitionTier>,
    pub name: Uuid,
    pub timestamp: DateTime<Utc>,
    pub loc_name: String,
    pub loc_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDefinitionTier {
    pub tier: u8,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub skills: Vec<SkillItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillItem {
    pub name: String,
    pub i18n_name: String,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub levels: Vec<SkillItemLevel>,
    pub loc_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillItemLevel {
    pub level: u8,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub cost: Map<String, Value>,
    pub bonus: Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterClasses {
    pub list: Vec<Class>,
    pub skill_definitions: &'static [SkillDefinition],
}

// Everyone contains their own instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub i18n_name: String,
    pub i18n_description: String,
    pub level_name: Uuid,
    pub prestige_level_name: Uuid,
    pub points: Map<String, Value>,
    // Default skill trees to clone from
    pub skill_trees: Vec<SkillTreeEntry>,
    pub attributes: Map<String, Value>,
    pub bonus: Map<String, Value>,
    pub custom_attributes: Map<String, Value>,
    pub unlocked: bool,
    pub default_equipments: Vec<CharacterEquipment>,
    pub default_customization: Map<String, Value>,
    pub name: Uuid,
    pub inventory_namespace: String,
    pub autogenerate_inventory_namespace: bool,
    pub initial_active_candidate: bool,
    pub item_link: String, //0:{ITEM_ID}
    pub default_namespace: String,
    pub loc_name: String,
    pub loc_description: String,
}

// Everyone contains their own instance
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockedCharacters {
    pub active_character_id: Uuid,
    pub list: Vec<Character>,
}
