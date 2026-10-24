use std::ops::DerefMut;

use anyhow::{Context, anyhow};
use pocket_ark_database::{
    DbTransaction,
    dto::{
        character::{CreateCharacterDto, PlayStats},
        users::UserId,
    },
    repositories::characters::CharactersRepository,
};
use pocket_ark_definitions::{
    classes::{Classes, PointMap},
    items::ItemName,
    level_tables::{LevelTables, ProgressionXp},
};
use uuid::Uuid;

/// The default level of characters when acquiring them
const DEFAULT_CHARACTER_LEVEL: u32 = 1;

pub fn acquire_character_dto(
    classes: &Classes,
    level_tables: &LevelTables,

    user_id: UserId,
    item: &ItemName,
) -> anyhow::Result<CreateCharacterDto> {
    let class = classes
        .by_item(item)
        .ok_or(anyhow!("Missing class for character item"))?;

    let level = DEFAULT_CHARACTER_LEVEL;
    let xp: ProgressionXp = level_tables
        .by_name(&class.level_name)
        .context("Missing character level table")?
        .get_xp_values(level)
        .context("Invalid character level provided")?
        .into();

    Ok(CreateCharacterDto {
        character_id: Uuid::new_v4(),
        user_id,
        class_name: class.name,
        level,
        xp,
        points: PointMap::default_total(),
        skill_trees: class.skill_trees.clone(),
        attributes: class.attributes.clone(),
        bonus: class.bonus.clone(),
        equipments: class.default_equipments.clone(),
        customization: class.default_customization.clone(),
        promotion: 0,
        points_spent: PointMap::default_spent(),
        points_granted: PointMap::default_spent(),
        play_stats: PlayStats::default(),
    })
}

/// Handles the initialization of a character after an item for
/// that character has been acquired
pub async fn acquire_item_character(
    classes: &Classes,
    level_tables: &LevelTables,

    db: &mut DbTransaction<'_>,
    user_id: UserId,
    item: &ItemName,
) -> anyhow::Result<()> {
    let class = classes
        .by_item(item)
        .ok_or(anyhow!("Missing class for character item"))?;

    // User already has the character unlocked
    if let Some(_existing) =
        CharactersRepository::get_by_user_by_class(db.deref_mut(), user_id, class.name).await?
    {
        // TODO: Getting the same character as a reward again adds 4 skill points for card rank II and IV and 5 points for VI, VII, and X
        return Ok(());
    }

    let dto = acquire_character_dto(classes, level_tables, user_id, item)?;
    CharactersRepository::create(db.deref_mut(), dto).await?;
    Ok(())
}
