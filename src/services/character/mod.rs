use std::{collections::HashMap, process::exit};

use log::error;
use uuid::Uuid;

use crate::http::models::character::{Class, LevelTable, SkillDefinition};

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
    pub fn new() {
        let classes = match ClassLookup::new() {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to parse class definitions: {}", err);
                exit(1);
            }
        };
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
    class_by_item: HashMap<String, usize>,
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
                Some((_, item)) => item.to_string(),
                None => {
                    error!(
                        "Class {} has an invalid item link: '{}'",
                        class.name, class.item_link
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

    pub fn by_item(&self, item: &str) -> Option<&Class> {
        let index = self.class_by_item.get(item).copied()?;
        let class = &self.classes[index];
        Some(class)
    }
}
