use super::HttpError;
use crate::{
    database::entity::{Character, SharedData, characters::CharacterId},
    definitions::{
        classes::{CharacterEquipment, Class, CustomizationEntry},
        level_tables::LevelTable,
        skills::{SkillDefinition, SkillTree},
    },
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CharactersError {
    #[error("Character not found")]
    NotFound,
}

impl HttpError for CharactersError {
    fn status(&self) -> StatusCode {
        match self {
            CharactersError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharactersResponse {
    pub list: Vec<Character>,
    #[serde(flatten)]
    pub shared_data: SharedData,
}

/// Request to update customization entries, provides a
/// map of the entries to update
#[derive(Debug, Deserialize)]
pub struct UpdateCustomizationRequest {
    pub customization: HashMap<String, CustomizationEntryUpdate>,
}

/// Request to update a characters skill trees by diffing
/// them with the provided entries
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSkillTreesRequest {
    pub skill_trees: Vec<SkillTree>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterResponse {
    #[serde(flatten)]
    pub character: Character,
    #[serde(flatten)]
    pub shared_data: SharedData,
}

#[derive(Debug, Serialize)]
pub struct CharacterLevelTables {
    pub list: &'static [LevelTable],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterEquipmentList {
    pub list: Vec<CharacterEquipment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterClasses {
    pub list: Vec<ClassWithState>,
    pub skill_definitions: &'static [SkillDefinition],
}

/// Class definition with an additional unlocked state
#[derive(Debug, Serialize)]
pub struct ClassWithState {
    #[serde(flatten)]
    pub class: &'static Class,
    pub unlocked: bool,
}

/// List of unlocked characters (Usage not yet known)
#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockedCharacters {
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub active_character_id: Option<CharacterId>,
    pub list: Vec<Character>,
}

/// Request to update a customization entry values
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationEntryUpdate {
    pub value_x: f32,
    pub value_y: f32,
    pub value_z: f32,
    pub value_w: f32,
    #[serde(rename = "type")]
    pub ty: u32,
    pub param_id: u32,
}

impl From<CustomizationEntryUpdate> for CustomizationEntry {
    fn from(value: CustomizationEntryUpdate) -> Self {
        Self {
            value_x: value.value_x,
            value_y: value.value_y,
            value_z: value.value_z,
            value_w: value.value_w,
            ty: value.ty,
            param_id: value.param_id,
        }
    }
}
