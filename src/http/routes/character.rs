use crate::{
    database::entity::{
        characters::{self, CharacterId, CustomizationMap, EquipmentList},
        Character, ClassData, SharedData,
    },
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            character::{
                CharacterClasses, CharacterEquipmentList, CharacterLevelTables, CharacterResponse,
                CharactersResponse, ClassWithState, UnlockedCharacters, UpdateCustomizationRequest,
                UpdateSkillTreesRequest,
            },
            RawHttpError,
        },
    },
    services::character::SkillDefinition,
    state::App,
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
) -> Result<Json<CharactersResponse>, RawHttpError> {
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
) -> Result<Json<CharacterResponse>, RawHttpError> {
    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

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
) -> Result<StatusCode, RawHttpError> {
    debug!("Requested set active character: {}", character_id);

    // TODO: validate the character is actually owned
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
) -> Result<Json<CharacterEquipmentList>, RawHttpError> {
    debug!("Requested character equip: {}", character_id);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

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
) -> Result<StatusCode, RawHttpError> {
    debug!("Update character equipment: {} - {:?}", character_id, req);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

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
) -> Result<StatusCode, RawHttpError> {
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
) -> Result<StatusCode, RawHttpError> {
    debug!(
        "Update character customization: {} - {:?}",
        character_id, req
    );

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

    let map = req
        .customization
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect();

    let mut character = character.into_active_model();
    character.customization = ActiveValue::Set(CustomizationMap(map));
    let _ = character.update(&db).await;

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
) -> Result<Json<CharacterEquipmentList>, RawHttpError> {
    // TODO: Currently just gives current equip maybe save previous list

    debug!("Requested character equip history: {}", character_id);

    let character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

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
) -> Result<Json<Character>, RawHttpError> {
    debug!("Req update skill tree: {} {:?}", character_id, req);

    let mut character = user
        .find_related(characters::Entity)
        .filter(characters::Column::Id.eq(character_id))
        .one(&db)
        .await?
        .ok_or(RawHttpError::new(
            "Character not found",
            StatusCode::NOT_FOUND,
        ))?;

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
                        par.skills.insert(key, value);
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
) -> Result<Json<CharacterClasses>, RawHttpError> {
    let services = App::services();

    let class_data = ClassData::all(&db, &user).await?;

    // Combine classes with unlocked class data states
    let list: Vec<ClassWithState> = services
        .character
        .classes
        .list()
        .iter()
        .map(|class| {
            let unlocked = class_data
                .iter()
                .find(|class_data| class_data.class_name == class.name)
                .is_some_and(|class_data| class_data.unlocked);

            ClassWithState { class, unlocked }
        })
        .collect();

    let skill_definitions: &'static [SkillDefinition] = &services.character.skills;

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
    let services = App::services();
    Json(CharacterLevelTables {
        list: &services.character.level_tables,
    })
}

/// POST /character/unlocked
///
/// Returns a list of unlocked characters?
pub async fn character_unlocked(
    Auth(user): Auth,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<UnlockedCharacters>, RawHttpError> {
    debug!("Unlocked request");
    let shared_data = SharedData::get(&db, &user).await?;

    // TODO: Should actually handle creating definitions for an unlocked character if they
    // are not already created

    Ok(Json(UnlockedCharacters {
        active_character_id: shared_data.active_character_id,
        list: vec![],
    }))
}
