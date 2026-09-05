use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    database::dto::users::UserId,
    definitions::{
        classes::{CharacterAttributes, CharacterEquipment, ClassName, CustomizationMap, PointMap},
        level_tables::ProgressionXp,
        skills::SkillTree,
    },
    utils::models::Sku,
};

pub type CharacterId = i64;

#[derive(Debug, FromRow)]
pub struct CharacterDto {
    /// Unique ID of the character
    pub id: CharacterId,
    pub character_id: Uuid,
    /// ID of the user that owns this character
    pub user_id: UserId,
    /// Name of the class definition this character belongs to
    pub class_name: ClassName,
    /// The current level of the characters
    pub level: u32,
    /// XP progression data associated with this character
    #[sqlx(json)]
    pub xp: ProgressionXp,
    /// Number of promotions this character has been given
    pub promotion: u32,
    /// Mapping for available point allocations
    #[sqlx(json)]
    pub points: PointMap,
    /// Mapping for spent point allocations
    #[sqlx(json)]
    pub points_spent: PointMap,
    /// Mapping for total points given
    #[sqlx(json)]
    pub points_granted: PointMap,
    /// Skill tree progression data
    #[sqlx(json)]
    pub skill_trees: Vec<SkillTree>,
    /// Character attributes
    #[sqlx(json)]
    pub attributes: CharacterAttributes,
    /// Character bonus data
    #[sqlx(json)]
    pub bonus: serde_json::Map<String, serde_json::Value>,
    /// Character equipment list
    #[sqlx(json)]
    pub equipments: Vec<CharacterEquipment>,
    /// Character customization data
    #[sqlx(json)]
    pub customization: CustomizationMap,
    /// Character usage stats
    #[sqlx(json)]
    pub play_stats: PlayStats,
    /// Last time this character was used
    pub last_used: Option<DateTime<Utc>>,
    /// Whether this character is promotable
    pub promotable: bool,
}
#[derive(Debug, Default)]
pub struct CreateCharacterDto {
    pub character_id: Uuid,
    pub user_id: UserId,
    pub class_name: ClassName,
    pub level: u32,
    pub xp: ProgressionXp,
    pub promotion: u32,
    pub points: PointMap,
    pub points_spent: PointMap,
    pub points_granted: PointMap,
    pub skill_trees: Vec<SkillTree>,
    pub attributes: CharacterAttributes,
    pub bonus: serde_json::Map<String, serde_json::Value>,
    pub equipments: Vec<CharacterEquipment>,
    pub customization: CustomizationMap,
    pub play_stats: PlayStats,
}

impl Serialize for CharacterDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state: <S as Serializer>::SerializeStruct =
            Serializer::serialize_struct(serializer, "Character", 19)?;
        state.serialize_field("characterId", &self.character_id.to_string())?;
        state.serialize_field("sku", &Sku)?;
        state.serialize_field("characterClassName", &self.class_name)?;
        state.serialize_field("name", &self.class_name)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("xp", &self.xp)?;
        state.serialize_field("promotion", &self.promotion)?;
        state.serialize_field("points", &self.points)?;
        state.serialize_field("pointsSpent", &self.points_spent)?;
        state.serialize_field("pointsGranted", &self.points_granted)?;
        state.serialize_field("skillTrees", &self.skill_trees)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("bonus", &self.bonus)?;
        state.serialize_field("equipments", &self.equipments)?;
        state.serialize_field("customization", &self.customization)?;
        state.serialize_field("playStats", &self.play_stats)?;
        // Inventory namespace always appears to be "default"
        state.serialize_field("inventoryNamespace", "default")?;

        if self.last_used.is_some() {
            state.serialize_field("lastUsed", &self.last_used)?;
        }

        state.serialize_field("promotable", &self.promotable)?;
        state.end()
    }
}

/// TODO: Ensure this structure is complete
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayStats {
    pub career_duration: Option<f32>,
    /// Catch-all for unknown keys that haven't been determined yet
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
