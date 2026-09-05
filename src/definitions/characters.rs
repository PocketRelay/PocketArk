use std::ops::DerefMut;

use crate::database::DbTransaction;
use crate::{
    database::{
        dto::{
            character::{CharacterDto, CreateCharacterDto, PlayStats},
            users::UserDto,
        },
        repositories::characters::CharactersRepository,
    },
    definitions::{
        classes::{Classes, PointMap},
        items::ItemName,
        level_tables::{LevelTables, ProgressionXp},
    },
};
use anyhow::{Context, anyhow};
use sqlx::SqliteExecutor;
use uuid::Uuid;

/// Handles the initialization of a character after an item for
/// that character has been acquired
pub async fn acquire_item_character(
    db: &mut DbTransaction<'_>,
    user: &UserDto,
    item: &ItemName,
    classes: &Classes,
    level_tables: &LevelTables,
) -> anyhow::Result<()> {
    let class = classes
        .by_item(item)
        .ok_or(anyhow!("Missing class for character item"))?;

    // User already has the character unlocked
    if let Some(_existing) =
        CharactersRepository::get_by_user_by_class(db.deref_mut(), user.id, class.name).await?
    {
        // TODO: Getting the same character as a reward again adds 4 skill points for card rank II and IV and 5 points for VI, VII, and X

        return Ok(());
    }

    // Character is acquired at level 1
    let level = 1;

    // Get the current xp progression values
    let xp: ProgressionXp = level_tables
        .by_name(&class.level_name)
        .context("Missing character level table")?
        .get_xp_values(level)
        .context("Invalid character level provided")?
        .into();

    let points: PointMap = PointMap {
        skill_points: Some(5),
    };
    let skill_trees = class.skill_trees.clone();
    let attributes = class.attributes.clone();
    let bonus = class.bonus.clone();
    let equipments = class.default_equipments.clone();
    let customization = class.default_customization.clone();

    CharactersRepository::create(
        db.deref_mut(),
        CreateCharacterDto {
            character_id: Uuid::new_v4(),
            user_id: user.id,
            class_name: class.name,
            level,
            xp,
            points,
            skill_trees,
            attributes,
            bonus,
            equipments,
            customization,
            promotion: 0,
            points_spent: PointMap::default_spent(),
            points_granted: PointMap::default_spent(),
            play_stats: PlayStats::default(),
        },
    )
    .await?;

    Ok(())
}
