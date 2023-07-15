use std::{collections::HashMap, hash::Hash, process::exit};

use crate::http::models::{
    character::{Class, LevelTable, SkillDefinition},
    inventory::ItemDefinition,
};
use log::{debug, error};
use uuid::Uuid;

/// Definitions for all the items
pub const INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

pub const SKILL_DEFINITIONS: &str = include_str!("../../resources/data/skillDefinitions.json");
pub const CLASS_DEFINITIONS: &str = include_str!("../../resources/data/characterClasses.json");
pub const LEVEL_TABLE_DEFINITIONS: &str =
    include_str!("../../resources/data/characterLevelTables.json");

pub struct Definitions {
    pub inventory: LookupList<String, ItemDefinition>,
    pub skills: LookupList<Uuid, SkillDefinition>,
    pub classes: LookupList<Uuid, Class>,
    pub level_tables: LookupList<Uuid, LevelTable>,
}

pub type LevelTables = LookupList<Uuid, LevelTable>;

impl Definitions {
    pub fn load() -> Self {
        debug!("Loading definitions");

        let inventory = Self::load_inventory();
        let skills = Self::load_skills();
        let classes = Self::load_classes();
        let level_tables = Self::load_level_tables();

        Self {
            inventory,
            skills,
            classes,
            level_tables,
        }
    }

    fn load_inventory() -> LookupList<String, ItemDefinition> {
        let list: Vec<ItemDefinition> = match serde_json::from_str(INVENTORY_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load inventory definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} inventory item definition(s)", list.len());
        LookupList::create(list, |value| value.name.to_string())
    }

    fn load_skills() -> LookupList<Uuid, SkillDefinition> {
        let list: Vec<SkillDefinition> = match serde_json::from_str(SKILL_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load skill definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} skill definition(s)", list.len());

        LookupList::create(list, |value| value.name)
    }
    fn load_classes() -> LookupList<Uuid, Class> {
        let list: Vec<Class> = match serde_json::from_str(CLASS_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to class skill definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} class definition(s)", list.len());

        LookupList::create(list, |value| {
            let (_left, right) = value
                .item_link
                .split_once(':')
                .expect("Failed to parse class");

            Uuid::try_parse(right).expect("Failed to parse class UUID")
        })
    }
    fn load_level_tables() -> LookupList<Uuid, LevelTable> {
        let list: Vec<LevelTable> = match serde_json::from_str(LEVEL_TABLE_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to level table definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} level table definition(s)", list.len());

        LookupList::create(list, |value| value.name)
    }
}

pub struct LookupList<K, V> {
    lookup: HashMap<K, usize>,
    list: Vec<V>,
}

impl<K, V> LookupList<K, V>
where
    K: Hash + PartialEq + Eq,
{
    pub fn create<F>(list: Vec<V>, key_fn: F) -> LookupList<K, V>
    where
        F: Fn(&V) -> K,
    {
        let mut lookup = HashMap::with_capacity(list.len());
        list.iter().enumerate().for_each(|(index, value)| {
            let key = key_fn(value);
            lookup.insert(key, index);
        });
        LookupList { lookup, list }
    }

    pub fn list(&self) -> &[V] {
        &self.list
    }

    pub fn lookup(&self, key: &K) -> Option<&V> {
        let index = self.lookup.get(key)?;
        let value = &self.list[*index];
        Some(value)
    }
}
