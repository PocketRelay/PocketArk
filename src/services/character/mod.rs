use crate::utils::models::{LocaleName, LocaleNameWithDesc};
use chrono::{DateTime, Utc};
use log::{debug, error};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
use std::{collections::HashMap, process::exit, str::FromStr};
use uuid::Uuid;

const CLASS_DEFINITIONS: &str = include_str!("../../resources/data/characterClasses.json");
const SKILL_DEFINITIONS: &str = include_str!("../../resources/data/skillDefinitions.json");
const LEVEL_TABLE_DEFINITIONS: &str =
    include_str!("../../resources/data/characterLevelTables.json");

pub struct CharacterService {
    pub skills: Vec<SkillDefinition>,
    pub classes: ClassLookup,
    pub level_tables: Vec<LevelTable>,
}

impl CharacterService {
    pub fn new() -> Self {
        let classes = match ClassLookup::new() {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to parse class definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} class definition(s)", classes.classes.len());

        let skills: Vec<SkillDefinition> = match serde_json::from_str(SKILL_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to parse skill definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} skill definition(s)", skills.len());

        let level_tables: Vec<LevelTable> = match serde_json::from_str(LEVEL_TABLE_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to parse level table definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} level table definition(s)", level_tables.len());

        Self {
            classes,
            skills,
            level_tables,
        }
    }

    pub fn level_table(&self, name: &Uuid) -> Option<&LevelTable> {
        self.level_tables
            .iter()
            .find(|level_table| level_table.name.eq(name))
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
    fn new() -> serde_json::Result<Self> {
        let classes: Vec<Class> = serde_json::from_str(CLASS_DEFINITIONS)?;
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
    pub level_name: Uuid,
    pub prestige_level_name: Uuid,
    pub points: Map<String, Value>,
    // Default skill trees to clone from
    pub skill_trees: Vec<SkillTreeEntry>,
    pub attributes: Map<String, Value>,
    pub bonus: Map<String, Value>,
    pub custom_attributes: Map<String, Value>,
    pub default_equipments: Vec<CharacterEquipment>,
    pub default_customization: HashMap<String, CustomizationEntry>,
    pub name: Uuid,
    pub inventory_namespace: String,
    pub autogenerate_inventory_namespace: bool,
    pub initial_active_candidate: bool,
    pub item_link: String, //0:{ITEM_ID}
    pub default_namespace: String,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTable {
    pub name: Uuid,
    pub table: Vec<LevelTableEntry>,
    pub custom_attributes: HashMap<String, Value>,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}

impl LevelTable {
    pub fn get_entry_xp(&self, level: u32) -> Option<u32> {
        self.table
            .iter()
            .find(|value| value.level == level)
            .map(|value| value.xp)
    }

    /// Computes the new xp and level values from the provided
    /// initial xp, level and the earned xp amount. Uses the
    /// current level table
    ///
    /// # Arguments
    /// * xp - The starting xp value  
    /// * level - The starting level value
    /// * xp_earned - The total xp earned
    pub fn compute_leveling(&self, mut xp: Xp, mut level: u32, xp_earned: u32) -> (Xp, u32) {
        xp.current = xp.current.saturating_add(xp_earned);

        while xp.current >= xp.next {
            let next_xp = match self.get_entry_xp(level + 1) {
                Some(value) => value,
                None => break,
            };

            level += 1;

            // Subtract the old next amount from earnings
            xp.current -= xp.next;

            // Assign new next and last values
            xp.last = xp.next;
            xp.next = next_xp;
        }

        (xp, level)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTableEntry {
    pub level: u32,
    pub xp: u32,
    pub rewards: HashMap<String, f64>,
    pub custom_attributes: HashMap<String, Value>,
}

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
