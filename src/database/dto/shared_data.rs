use std::collections::HashMap;

use crate::utils::serialize::{deserialize_f64_u32, serialize_f64_u32};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    database::dto::users::UserId,
    definitions::{
        classes::CharacterEquipment,
        i18n::{I18nDescription, I18nName},
        level_tables::ProgressionXp,
    },
};

#[derive(Debug, FromRow, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct SharedDataDto {
    #[serde(skip)]
    pub user_id: UserId,

    pub active_character_id: Option<Uuid>,

    #[sqlx(json)]
    pub shared_stats: SharedStats,
    #[sqlx(json)]
    pub shared_equipment: CharacterSharedEquipment,
    #[sqlx(json)]
    pub shared_progression: Vec<SharedProgression>,
}

#[derive(Debug)]
pub struct CreateSharedDataDto {
    pub user_id: UserId,
    pub shared_stats: SharedStats,
    pub shared_equipment: CharacterSharedEquipment,
    pub shared_progression: Vec<SharedProgression>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedStats {
    /// The pathfinder rating for the user. Stored as a integer
    /// but serialized as a decimal
    #[serde(
        deserialize_with = "deserialize_f64_u32",
        serialize_with = "serialize_f64_u32"
    )]
    pub pathfinder_rating: u32,
    /// Other shared stats
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSharedEquipment {
    pub list: Vec<CharacterEquipment>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SharedProgression {
    pub name: Uuid,
    #[serde(flatten)]
    pub i18n_name: I18nName,
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
    pub level: u32,
    pub xp: ProgressionXp,
}
