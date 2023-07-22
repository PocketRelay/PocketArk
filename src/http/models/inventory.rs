use crate::database::entity::{Currency, InventoryItem};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct InventoryResponse {
    pub items: Vec<InventoryItem>,
    pub definitions: Vec<&'static ItemDefinition>,
}

#[derive(Debug, Serialize)]
pub struct InventoryDefinitions {
    pub total_count: usize,
    pub list: &'static [ItemDefinition],
}

#[derive(Debug, Deserialize)]
pub struct InventorySeenList {
    pub list: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryConsumeRequest {
    pub items: Vec<ConsumeTarget>,
    pub namespace: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeTarget {
    pub item_id: Uuid,
    pub target_id: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefinition {
    pub name: String,
    pub i18n_name: String,
    pub i18n_description: Option<String>,
    pub loc_name: Option<String>,
    pub loc_description: Option<String>,
    pub custom_attributes: HashMap<String, Value>,
    #[serialize_always]
    pub secret: Option<Value>,
    pub category: String,
    pub attachable_categories: Vec<String>,
    pub rarity: Option<String>,
    pub droppable: Option<bool>,
    pub cap: Option<u32>,

    /// Name of definition that this item depends on
    pub unlock_definition: Option<String>,

    pub on_consume: Option<Vec<Value>>,
    pub on_add: Option<Vec<Value>>,
    pub on_remove: Option<Vec<Value>>,
    pub restrictions: Option<String>,
    pub default_namespace: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResult {
    pub previous_xp: u32,
    pub xp: u32,
    pub xp_gained: u32,
    pub previous_level: u32,
    pub level: u32,
    pub level_up: bool,
    pub character_class_name: Option<Uuid>,
    pub challenges_updated_count: u32,
    pub challenges_completed_count: u32,
    pub challenges_updated: Vec<Value>,
    pub updated_challenge_ids: Vec<Value>,
    pub news_triggered: u32,
    pub currencies: Vec<Currency>,
    pub currency_earned: Vec<Currency>,
    pub items_earned: Vec<InventoryItem>,
    pub item_definitions: Vec<&'static ItemDefinition>,
    pub entitlements_granted: Vec<Value>,
    pub prestige_progression_map: Map<String, Value>,
}
