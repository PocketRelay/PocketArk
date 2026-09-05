use std::mem::swap;

use crate::{
    database::{
        DbPool,
        dto::{character::CharacterDto, shared_data::CharacterSharedEquipment},
        repositories::{characters::CharactersRepository, shared_data::SharedDataRepository},
    },
    definitions::{
        classes::{ClassName, Classes, CustomizationMap},
        level_tables::LevelTables,
        skills::{SkillDefinition, Skills},
    },
    http::{
        middleware::{JsonDump, user::Auth},
        models::{
            character::*,
            errors::{DynHttpError, HttpResult},
        },
    },
};
use axum::{Extension, Json, extract::Path};
use hyper::StatusCode;
use log::debug;
use uuid::Uuid;

/// GET /characters
pub async fn get_characters(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<CharactersResponse> {
    let list = CharactersRepository::get_by_user(&db, user.id).await?;
    let shared_data = SharedDataRepository::get_by_user(&db, user.id).await?;

    Ok(Json(CharactersResponse { list, shared_data }))
}

/// GET /character/:id
///
/// Gets the definition and details for the character of the provided ID
pub async fn get_character(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<CharacterResponse> {
    let character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    let shared_data = SharedDataRepository::get_by_user(&db, user.id).await?;

    Ok(Json(CharacterResponse {
        character,
        shared_data,
    }))
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Requested set active character: {}", character_id);

    // Ensure the player actually owns the character
    _ = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound);

    // Update the shared data
    SharedDataRepository::set_user_active_character(&db, user.id, character_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment
///
/// Gets the current equipment of the provided character
pub async fn get_character_equip(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<CharacterEquipmentList> {
    debug!("Requested character equip: {}", character_id);

    let character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments,
    }))
}

/// PUT /character/:id/equipment
///
/// Updates the equipment for the provided character using
/// the provided equipment list
pub async fn update_character_equip(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
    JsonDump(req): JsonDump<CharacterEquipmentList>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Update character equipment: {} - {:?}", character_id, req);

    let character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    CharactersRepository::set_equipments(&db, character.id, req.list).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/equipment/shared
///
/// Updates share character equipment
pub async fn update_shared_equip(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
    JsonDump(req): JsonDump<CharacterEquipmentList>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Update shared equipment: {:?}", req);

    SharedDataRepository::set_user_shared_equipment(
        &db,
        user.id,
        CharacterSharedEquipment { list: req.list },
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/:id/customization
///
/// Updates the customization settings for a character
pub async fn update_character_customization(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
    JsonDump(req): JsonDump<UpdateCustomizationRequest>,
) -> Result<StatusCode, DynHttpError> {
    debug!(
        "Update character customization: {} - {:?}",
        character_id, req
    );

    let mut character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    // Swap the customization map for an empty one so we can edit it
    let mut customization = CustomizationMap::default();
    swap(&mut customization, &mut character.customization);

    // Update the customization with the request values
    req.customization
        .into_iter()
        .for_each(|(key, value)| customization.set(key, value.into()));

    // Update the stored customization

    CharactersRepository::set_customization(&db, character.id, customization).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment/history
///
/// Obtains the history of the characters previous
/// equipment
pub async fn get_character_equip_history(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<CharacterEquipmentList> {
    // TODO: Currently just gives current equip maybe save previous list

    debug!("Requested character equip history: {}", character_id);

    let character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments,
    }))
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree(
    Path(character_id): Path<Uuid>,
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
    JsonDump(req): JsonDump<UpdateSkillTreesRequest>,
) -> HttpResult<CharacterDto> {
    debug!("Req update skill tree: {} {:?}", character_id, req);

    let mut character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    // TODO: Calculate skill requirement and ensure user can afford it, update
    // associated points fields

    // TODO: Clean this up and properly diff the trees
    req.skill_trees.into_iter().for_each(|tree| {
        let par = character
            .skill_trees
            .iter_mut()
            .find(|value| value.name == tree.name);
        if let Some(par) = par {
            for entry in tree.tree {
                let par = par.tree.iter_mut().find(|value| value.tier == entry.tier);
                if let Some(par) = par {
                    for (key, value) in entry.skills {
                        par.set_skill(key, value);
                    }
                }
            }
        }
    });

    // TODO: Update available skill points
    CharactersRepository::set_skill_trees(&db, character.id, character.skill_trees).await?;

    let character = CharactersRepository::get_by_user_by_id(&db, user.id, character_id)
        .await?
        .ok_or(CharactersError::NotFound)?;

    Ok(Json(character))
}

/// GET /character/classes
pub async fn get_classes(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<CharacterClasses> {
    // Get the unlocked classes
    let unlocked_classes: Vec<ClassName> =
        CharactersRepository::get_user_classes(&db, user.id).await?;

    let class_definitions = Classes::get();

    // Combine classes with unlocked class data states
    let list: Vec<ClassWithState> = class_definitions
        .all()
        .iter()
        .map(|class| {
            let unlocked = unlocked_classes.contains(&class.name);

            ClassWithState { class, unlocked }
        })
        .collect();

    let skill_definitions = Skills::get();
    let skill_definitions: &'static [SkillDefinition] = &skill_definitions.values;

    Ok(Json(CharacterClasses {
        list,
        skill_definitions,
    }))
}

/// GET /character/levelTables
///
/// Contains definitions for rewards at each level of character
/// progression
pub async fn get_level_tables() -> Json<CharacterLevelTables> {
    let level_tables = LevelTables::get();

    Json(CharacterLevelTables {
        list: &level_tables.values,
    })
}

/// POST /character/unlocked
///
/// Returns a list of unlocked characters?
pub async fn character_unlocked(
    Auth(user): Auth,
    Extension(db): Extension<DbPool>,
) -> HttpResult<UnlockedCharacters> {
    debug!("Unlocked request");
    let shared_data = SharedDataRepository::get_by_user(&db, user.id).await?;

    // TODO: Should actually handle creating definitions for an unlocked character if they
    // are not already created

    Ok(Json(UnlockedCharacters {
        active_character_id: shared_data.active_character_id,
        list: vec![],
    }))
}
