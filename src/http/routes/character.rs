use std::collections::HashMap;

use crate::{
    database::entity::{
        characters::{CustomizationMap, EquipmentList},
        Character, UserEntity,
    },
    http::models::{
        character::{
            CharacterClasses, CharacterEquipmentList, CharacterResponse, CharactersResponse, Class,
            SkillDefinition, UnlockedCharacters, UpdateCustomizationRequest,
            UpdateSkillTreesRequest,
        },
        HttpError, RawJson,
    },
    state::App,
};
use axum::{extract::Path, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

/// GET /characters
pub async fn get_characters() -> Result<Json<CharactersResponse>, HttpError> {
    let db = App::database();

    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;
    let list = user
        .find_related(crate::database::entity::characters::Entity)
        .all(db)
        .await?;

    let shared_data = user.get_shared_data(db).await?;

    Ok(Json(CharactersResponse { list, shared_data }))
}

/// GET /character/:id
///
/// Gets the defintion and details for the character of the provided ID
pub async fn get_character(
    Path(character_id): Path<Uuid>,
) -> Result<Json<CharacterResponse>, HttpError> {
    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    let shared_data = user.get_shared_data(db).await?;

    Ok(Json(CharacterResponse {
        character,
        shared_data,
    }))
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active(Path(character_id): Path<Uuid>) -> Result<StatusCode, HttpError> {
    debug!("Requested set active character: {}", character_id);
    let db = App::database();

    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;
    let shared_data = user.get_shared_data(db).await?;
    // TODO: validate the character is actually owned
    let _ = shared_data.set_active_character(character_id, db).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment
///
/// Gets the current equipment of the provided character
pub async fn get_character_equip(
    Path(character_id): Path<Uuid>,
) -> Result<Json<CharacterEquipmentList>, HttpError> {
    debug!("Requested character equip: {}", character_id);
    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments.0,
    }))
}

/// PUT /character/:id/equipment
///
/// Updates the equipment for the provided character using
/// the provided equipment list
pub async fn update_character_equip(
    Path(character_id): Path<Uuid>,
    Json(req): Json<CharacterEquipmentList>,
) -> Result<StatusCode, HttpError> {
    debug!("Update character equipment: {} - {:?}", character_id, req);

    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    let mut character = character.into_active_model();
    character.equipments = ActiveValue::Set(EquipmentList(req.list));
    let _ = character.update(db).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/equipment/shared
///
/// Updates share character equipment
pub async fn update_shared_equip(
    Json(req): Json<CharacterEquipmentList>,
) -> Result<StatusCode, HttpError> {
    debug!("Update shared equipment: {:?}", req);

    let db = App::database();

    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;
    let shared_data = user.get_shared_data(db).await?;

    let mut shared_data = shared_data.into_active_model();
    shared_data.shared_equipment = ActiveValue::Set(
        crate::database::entity::shared_data::CharacterSharedEquipment { list: req.list },
    );
    let _ = shared_data.update(db).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/:id/customization
///
/// Updates the customization settings for a character
pub async fn update_character_customization(
    Path(character_id): Path<Uuid>,
    Json(req): Json<UpdateCustomizationRequest>,
) -> Result<StatusCode, HttpError> {
    debug!(
        "Update character customization: {} - {:?}",
        character_id, req
    );

    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    let map = req
        .customization
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect();

    let mut character = character.into_active_model();
    character.customization = ActiveValue::Set(CustomizationMap(map));
    let _ = character.update(db).await;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment/history
///
/// Obtains the history of the characters previous
/// equipment
pub async fn get_character_equip_history(
    Path(character_id): Path<Uuid>,
) -> Result<Json<CharacterEquipmentList>, HttpError> {
    // TODO: Currently just gives current equip maybe save previous list

    debug!("Requested character equip history: {}", character_id);
    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments.0,
    }))
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree(
    Path(character_id): Path<Uuid>,
    Json(req): Json<UpdateSkillTreesRequest>,
) -> Result<Json<Character>, HttpError> {
    debug!("Req update skill tree: {} {:?}", character_id, req);

    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let mut character = user
        .find_related(crate::database::entity::characters::Entity)
        .filter(crate::database::entity::characters::Column::CharacterId.eq(character_id))
        .one(db)
        .await?
        .ok_or(HttpError::new("Character not found", StatusCode::NOT_FOUND))?;

    // TODO: Clean this up and properly diff the trees
    req.skill_trees.into_iter().for_each(|tree| {
        let par = character
            .skill_trees
            .0
            .iter_mut()
            .find(|value| value.name == tree.name);
        if let Some(par) = par {
            for entry in tree.tree {
                let par = par.tree.iter_mut().find(|value| value.tier == entry.tier);
                if let Some(par) = par {
                    par.skills = entry.skills;
                }
            }
        }
    });

    let mut character = character.into_active_model();
    character.skill_trees =
        ActiveValue::Set(character.skill_trees.take().expect("Skill tree missing"));
    let character = character.update(db).await?;

    Ok(Json(character))
}

/// GET /character/classes
pub async fn get_classes() -> Result<Json<CharacterClasses>, HttpError> {
    let services = App::services();
    let skill_definitions: &'static [SkillDefinition] = &services.defs.skills.list;

    let mut list: Vec<Class> =
        serde_json::from_str(include_str!("../../resources/data/characterClasses.json"))
            .expect("Failed to parse characters");

    let db = App::database();
    // TODO: user should be found from session
    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;

    let class_data = user
        .find_related(crate::database::entity::class_data::Entity)
        .all(db)
        .await?;

    // Updating unlocks from classdata
    list.iter_mut().for_each(|value| {
        let data = class_data.iter().find(|v| v.name == value.name);
        if let Some(data) = data {
            value.unlocked = data.unlocked;
        }
    });

    Ok(Json(CharacterClasses {
        list,
        skill_definitions,
    }))
}

/// Definitions for rewards at each character level
static CHARACTER_LEVEL_TABLES: &str =
    include_str!("../../resources/data/characterLevelTables.json");

/// GET /character/levelTables
///
/// Contains definitions for rewards at each level of character
/// progression
pub async fn get_level_tables() -> RawJson {
    RawJson(CHARACTER_LEVEL_TABLES)
}

/// POST /character/unlocked
///
/// Returns a list of unlocked characters?
pub async fn character_unlocked() -> Result<Json<UnlockedCharacters>, HttpError> {
    debug!("Unlocked request");
    let db = App::database();

    let user = UserEntity::find_by_id(1u32)
        .one(db)
        .await?
        .ok_or(HttpError::new(
            "Server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;
    let shared_data = user.get_shared_data(db).await?;

    Ok(Json(UnlockedCharacters {
        active_character_id: shared_data.active_character_id,
        list: vec![],
    }))
}
