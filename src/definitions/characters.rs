use super::items::ItemName;
use super::{
    classes::{Classes, PointMap},
    level_tables::{LevelTables, ProgressionXp},
};
use crate::database::entity::{Character, User};
use anyhow::{anyhow, Context};
use sea_orm::ConnectionTrait;

/// Handles the initialization of a character after an item for
/// that character has been acquired
pub async fn acquire_item_character<C>(
    db: &C,
    user: &User,
    item: &ItemName,
    classes: &Classes,
    level_tables: &LevelTables,
) -> anyhow::Result<()>
where
    C: ConnectionTrait + Send,
{
    let class = classes
        .by_item(item)
        .ok_or(anyhow!("Missing class for character item"))?;

    // User already has the character unlocked
    if let Some(_existing) = Character::find_by_user_by_def(db, user, class.name).await? {
        // TODO: Getting the same character as a reward again adds 4 skill points for card rank II and IV and 5 points for VI, VII, and X

        return Ok(());
    }

    // Character is aquired at level 1
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
    let equipment = class.default_equipments.clone();
    let customization = class.default_customization.clone();

    Character::create(
        db,
        user,
        class.name,
        level,
        xp,
        points,
        skill_trees,
        attributes,
        bonus,
        equipment,
        customization,
    )
    .await?;

    Ok(())
}
