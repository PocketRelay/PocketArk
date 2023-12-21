use crate::utils::models::{LocaleName, LocaleNameWithDesc};
use anyhow::Context;
use chrono::{DateTime, Utc};
use log::{debug, error};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

use self::levels::LevelTables;

/// Class definitions (36)
const CLASS_DEFINITIONS: &str = include_str!("../../resources/data/characterClasses.json");
/// Skill definitions (64)
const SKILL_DEFINITIONS: &str = include_str!("../../resources/data/skillDefinitions.json");

pub mod class;
pub mod levels;

pub struct CharacterService {
    pub skills: Vec<SkillDefinition>,
    pub classes: ClassLookup,
    pub level_tables: LevelTables,
}

impl CharacterService {
    pub fn new() -> anyhow::Result<Self> {
        let classes = ClassLookup::new()?;

        debug!("Loaded {} class definition(s)", classes.classes.len());

        let skills: Vec<SkillDefinition> =
            serde_json::from_str(SKILL_DEFINITIONS).context("Failed to parse skill definitions")?;

        debug!("Loaded {} skill definition(s)", skills.len());

        let level_tables: LevelTables = LevelTables::new()?;

        Ok(Self {
            classes,
            skills,
            level_tables,
        })
    }
}

/// Lookup table over the classes list to allow
/// finding classes by name or by item link
pub struct ClassLookup {
    classes: Vec<Class>,
    class_by_name: HashMap<Uuid, usize>,
    class_by_item: HashMap<Uuid, usize>,
}

impl ClassLookup {
    fn new() -> anyhow::Result<Self> {
        let classes: Vec<Class> =
            serde_json::from_str(CLASS_DEFINITIONS).context("Failed to load class definitions")?;
        let mut class_by_name = HashMap::with_capacity(classes.len());
        let mut class_by_item = HashMap::with_capacity(classes.len());

        classes.iter().enumerate().for_each(|(index, class)| {
            class_by_name.insert(class.name, index);

            // Parse item link from class
            let item = match class.item_link.split_once(':') {
                Some((_, item)) => Uuid::from_str(item),
                None => {
                    error!(
                        "Class {} has an invalid item link: '{}'",
                        class.name, class.item_link
                    );
                    return;
                }
            };

            let item = match item {
                Ok(value) => value,
                Err(err) => {
                    error!(
                        "Class {} item link UUID invalid '{}': {}",
                        class.name, class.item_link, err
                    );
                    return;
                }
            };

            class_by_item.insert(item, index);
        });

        Ok(Self {
            classes,
            class_by_name,
            class_by_item,
        })
    }

    pub fn list(&self) -> &[Class] {
        self.classes.as_slice()
    }

    pub fn by_name(&self, name: &Uuid) -> Option<&Class> {
        let index = self.class_by_name.get(name).copied()?;
        let class = &self.classes[index];
        Some(class)
    }

    pub fn by_item(&self, item: &Uuid) -> Option<&Class> {
        let index = self.class_by_item.get(item).copied()?;
        let class = &self.classes[index];
        Some(class)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub name: Uuid,                //
    pub level_name: Uuid,          //
    pub prestige_level_name: Uuid, //
    pub item_link: String,         //0:{ITEM_ID}

    pub points: Map<String, Value>,
    // Default skill trees to clone from
    pub skill_trees: Vec<SkillTreeEntry>,
    pub attributes: Map<String, Value>,
    pub bonus: Map<String, Value>,
    pub custom_attributes: Map<String, Value>,
    pub default_equipments: Vec<CharacterEquipment>,
    pub default_customization: HashMap<String, CustomizationEntry>,
    pub inventory_namespace: String,
    pub autogenerate_inventory_namespace: bool,
    pub initial_active_candidate: bool,
    pub default_namespace: String,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc, //
}

impl Class {
    /// Attempts to parse the item link from the class `item_link` as the
    /// item [Uuid]
    pub fn linked_item(&self) -> Option<Uuid> {
        None
        // let ()

        // // Parse item link from class
        // let item = match class.item_link.split_once(':') {
        //     Some((_, item)) => Uuid::from_str(item),
        //     None => {
        //         error!(
        //             "Class {} has an invalid item link: '{}'",
        //             class.name, class.item_link
        //         );
        //         return;
        //     }
        // };

        // let item = match item {
        //     Ok(value) => value,
        //     Err(err) => {
        //         error!(
        //             "Class {} item link UUID invalid '{}': {}",
        //             class.name, class.item_link, err
        //         );
        //         return;
        //     }
        // };
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDefinition {
    pub name: Uuid,

    pub tiers: Vec<SkillDefinitionTier>,
    pub custom_attributes: Map<String, Value>,
    pub timestamp: DateTime<Utc>,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDefinitionTier {
    pub tier: u8,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub skills: Vec<SkillItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillItem {
    pub name: String,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub levels: Vec<SkillItemLevel>,

    #[serde(flatten)]
    pub locale: LocaleName,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillItemLevel {
    pub level: u8,
    pub custom_attributes: Map<String, Value>,
    pub unlock_conditions: Vec<Value>,
    pub cost: Map<String, Value>,
    pub bonus: Map<String, Value>,
}

#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
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
    pub name: String,
    pub attachments: Vec<String>,
}
