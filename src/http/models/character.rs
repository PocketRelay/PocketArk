use std::collections::HashMap;

use super::auth::Sku;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Character {
    pub character_id: Uuid,
    pub sku: Sku,
    pub character_class_name: Uuid,
    pub name: Uuid,
    pub level: u32,
    pub xp: CharacterXp,
    pub promotion: u32,
    pub points: HashMap<String, u32>,
    pub points_spent: HashMap<String, u32>,
    pub points_granted: HashMap<String, u32>,
    pub skill_trees: Vec<SkillTreeEntry>,
    pub attributes: Value,
    pub bonus: Value,
    pub equipments: Vec<CharacterEquipment>,
    pub customization: HashMap<String, CustomizationEntry>,
    pub play_stats: CharacterPlayStats,
    pub shared_stats: CharacterSharedStats,
    pub inventory_namespace: String,
    pub last_used: DateTime<Utc>,
    pub promotable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterXp {
    pub current: u32,
    pub last: u32,
    pub next: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SkillTreeEntry {
    pub name: Uuid,
    pub tree: Vec<SkillTreeTier>,
    pub timestamp: DateTime<Utc>,
    pub obsolete: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SkillTreeTier {
    pub tier: u32,
    pub skills: HashMap<String, u8>,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterEquipment {
    pub slot: String,
    pub name: Uuid,
    pub attachments: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomizationEntry {
    pub value_x: String,
    pub value_y: String,
    pub value_z: String,
    pub value_w: String,
    pub ty: String,
    pub param_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterPlayStats {
    pub career_duration: f64,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterSharedStats {
    pub pathfinder_rating: f64,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterSharedEquipment {
    pub list: Vec<SharedEquipmentItem>,
}

#[derive(Serialize, Deserialize)]
pub struct SharedEquipmentItem {
    pub slot: String,
    pub name: String,
    pub attachments: Vec<Uuid>,
}
