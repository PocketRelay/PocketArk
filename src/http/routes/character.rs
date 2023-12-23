use std::mem::swap;

use crate::{
    database::entity::{
        characters::{self, CharacterId, EquipmentList},
        Character, SharedData,
    },
    definitions::{
        classes::{ClassDefinitions, ClassName, CustomizationMap},
        level_tables::LevelTables,
        skills::{SkillDefinition, SkillDefinitions},
    },
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            character::*,
            errors::{DynHttpError, HttpResult},
        },
    },
};
use axum::{extract::Path, Extension, Json};
use hyper::StatusCode;
use log::debug;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, IntoActiveModel, ModelTrait,
    QueryFilter,
};

/// GET /characters
pub async fn get_characters(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<CharactersResponse> {
    let list = user.find_related(characters::Entity).all(&db).await?;
    let shared_data = SharedData::get(&db, &user).await?;

    Ok(Json(CharactersResponse { list, shared_data }))
}

/// GET /character/:id
///
/// Gets the defintion and details for the character of the provided ID
pub async fn get_character(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<CharacterResponse> {
    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(CharactersError::NotFound)?;

    let shared_data = SharedData::get(&db, &user).await?;

    Ok(Json(CharacterResponse {
        character,
        shared_data,
    }))
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Requested set active character: {}", character_id);

    // Ensure the player actually owns the character
    _ = Character::find_by_id_user(&db, &user, character_id)
        .await?
        .ok_or(CharactersError::NotFound);

    // Update the shared data
    let shared_data = SharedData::get(&db, &user).await?;
    shared_data.set_active_character(&db, character_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment
///
/// Gets the current equipment of the provided character
pub async fn get_character_equip(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<CharacterEquipmentList> {
    debug!("Requested character equip: {}", character_id);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(CharactersError::NotFound)?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments.0,
    }))
}

/// PUT /character/:id/equipment
///
/// Updates the equipment for the provided character using
/// the provided equipment list
pub async fn update_character_equip(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<CharacterEquipmentList>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Update character equipment: {} - {:?}", character_id, req);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(CharactersError::NotFound)?;

    let mut character = character.into_active_model();
    character.equipments = ActiveValue::Set(EquipmentList(req.list));
    let _ = character.update(&db).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/equipment/shared
///
/// Updates share character equipment
pub async fn update_shared_equip(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<CharacterEquipmentList>,
) -> Result<StatusCode, DynHttpError> {
    debug!("Update shared equipment: {:?}", req);
    let shared_data = SharedData::get(&db, &user).await?;
    shared_data.set_shared_equipment(&db, req.list).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /character/:id/customization
///
/// Updates the customization settings for a character
pub async fn update_character_customization(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<UpdateCustomizationRequest>,
) -> Result<StatusCode, DynHttpError> {
    debug!(
        "Update character customization: {} - {:?}",
        character_id, req
    );

    let mut character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
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
    _ = character.update_customization(&db, customization).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /character/:id/equipment/history
///
/// Obtains the history of the characters previous
/// equipment
pub async fn get_character_equip_history(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<CharacterEquipmentList> {
    // TODO: Currently just gives current equip maybe save previous list

    debug!("Requested character equip history: {}", character_id);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(CharactersError::NotFound)?;

    Ok(Json(CharacterEquipmentList {
        list: character.equipments.0,
    }))
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree(
    Path(character_id): Path<CharacterId>,
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
    JsonDump(req): JsonDump<UpdateSkillTreesRequest>,
) -> HttpResult<Character> {
    debug!("Req update skill tree: {} {:?}", character_id, req);

    let mut character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(CharactersError::NotFound)?;

    // TODO: Calculate skill requirement and ensure user can afford it, update
    // associated points fields

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
                    for (key, value) in entry.skills {
                        par.set_skill(key, value);
                    }
                }
            }
        }
    });

    // TODO: Update available skillpoints

    let mut character = character.into_active_model();
    character.skill_trees =
        ActiveValue::Set(character.skill_trees.take().expect("Skill tree missing"));
    let character = character.update(&db).await?;

    Ok(Json(character))
}

/// GET /character/classes
pub async fn get_classes(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<CharacterClasses> {
    // Get the unlocked classes
    let unlocked_classes: Vec<ClassName> = Character::get_user_classes(&db, &user).await?;

    let class_definitions = ClassDefinitions::get();

    // Combine classes with unlocked class data states
    let list: Vec<ClassWithState> = class_definitions
        .all()
        .iter()
        .map(|class| {
            let unlocked = unlocked_classes.contains(&class.name);

            ClassWithState { class, unlocked }
        })
        .collect();

    let skill_definitios = SkillDefinitions::get();
    let skill_definitions: &'static [SkillDefinition] = &skill_definitios.values;

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
    Extension(db): Extension<DatabaseConnection>,
) -> HttpResult<UnlockedCharacters> {
    debug!("Unlocked request");
    let shared_data = SharedData::get(&db, &user).await?;

    // TODO: Should actually handle creating definitions for an unlocked character if they
    // are not already created

    Ok(Json(UnlockedCharacters {
        active_character_id: shared_data.active_character_id,
        list: vec![],
    }))
}
