use std::{collections::HashMap, process::exit};

use crate::http::models::inventory::InventoryItemDefinition;
use log::{debug, error};

/// Definitions for all the items
pub static INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

pub struct Definitions {
    pub inventory: HashMap<String, InventoryItemDefinition>,
}

impl Definitions {
    pub async fn load() -> Self {
        debug!("Loading definitions");
        let data: Vec<InventoryItemDefinition> = match serde_json::from_str(INVENTORY_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load inventory definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} inventory item definition(s)", data.len());

        let inventory: HashMap<String, InventoryItemDefinition> = data
            .into_iter()
            .map(|value| (value.name.clone(), value))
            .collect();

        Self { inventory }
    }
}
