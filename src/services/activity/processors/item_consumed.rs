use crate::services::{
    activity::{ActivityResult, processors::ActivityProcessor},
    items::{AcquireItem, acquire_item, get_droppable_items},
};
use async_trait::async_trait;
use pocket_ark_database::{
    DbTransaction,
    dto::{inventory_items::InventoryItemDto, users::UserDto},
    repositories::inventory_items::InventoryItemsRepository,
};
use pocket_ark_definitions::{
    classes::Classes,
    items::{
        ItemName, Items,
        category::{BaseCategory, Category},
    },
    level_tables::LevelTables,
    packs::{GenerateError, ItemReward, Packs, RewardCollection},
};
use rand::{SeedableRng, rngs::StdRng};
use serde::Deserialize;
use serde_with::serde_as;
use std::ops::DerefMut;
use thiserror::Error;

/// Handles item consumption activity events, this handles producing rewards from
/// consuming things such as packs
pub struct ItemConsumedActivityProcessor;

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename = "camelCase")]
pub struct ItemConsumedAttributes {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    category: Category,
    definition_name: ItemName,

    // TODO: Investigated why this is unused and whether it needs to be used
    // (It's probably just always 1 due to how the game functions?)
    #[allow(unused)]
    count: u32,
}

/// Errors that can occur while processing a item
/// consumption
#[derive(Debug, Error)]
pub enum ItemConsumeError {
    #[error("Pack '{0}' not implemented")]
    PackNotImplemented(ItemName),

    #[error(transparent)]
    GenerateError(#[from] GenerateError),
}

#[async_trait]
impl ActivityProcessor for ItemConsumedActivityProcessor {
    type Event = ItemConsumedAttributes;

    async fn process_event(
        &self,
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: Self::Event,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        let item_definitions = Items::get();
        let classes = Classes::get();
        let level_tables = LevelTables::get();
        let packs = Packs::get();

        let mut rewards: RewardCollection = RewardCollection::default();

        match event.category.base() {
            BaseCategory::ItemPack => {
                // Find the item pack
                let pack = packs
                    .by_name(&event.definition_name)
                    .ok_or(ItemConsumeError::PackNotImplemented(event.definition_name))?;

                // Create a random generator
                let mut rng = StdRng::from_entropy();

                let required_names = item_definitions.droppable_required_names();

                // Collect the owned items
                let owned_items: Vec<InventoryItemDto> =
                    InventoryItemsRepository::get_by_user_by_definitions(
                        db.deref_mut(),
                        user.id,
                        &required_names,
                    )
                    .await?;

                // Get droppable items
                let droppable_items = get_droppable_items(item_definitions, &owned_items);

                // Generate collection of rewards
                pack.generate_rewards(&mut rng, &droppable_items, &mut rewards)
                    .map_err(ItemConsumeError::GenerateError)?;
            }

            BaseCategory::ApexPoints => {
                // TODO: Apex point awards
            }
            BaseCategory::StrikeTeamReward => {
                // TODO: Strike team rewards
            }
            BaseCategory::Consumable => {}
            BaseCategory::Boosters => {}
            BaseCategory::CapacityUpgrade => {}

            _ => {}
        }

        for reward in rewards.rewards {
            let ItemReward {
                definition,
                stack_size,
            } = reward;

            let item = acquire_item(
                classes,
                level_tables,
                db,
                user.id,
                AcquireItem {
                    definition,
                    stack_size,
                },
            )
            .await?;

            // Append the change to the result
            result.add_item(item, stack_size, definition);
        }

        Ok(())
    }
}
