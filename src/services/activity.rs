//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use crate::{
    database::{
        DbTransaction,
        dto::{
            challenge_progress::{ChallengeCounterName, ChallengeId},
            currency::CurrencyDto,
            inventory_items::{CreateInventoryItemDto, InventoryItemDto},
            users::UserDto,
        },
        repositories::{currency::CurrencyRepository, inventory_items::InventoryItemsRepository},
    },
    definitions::{
        activity::{ActivityEvent, ActivityName},
        classes::Classes,
        items::{
            ItemDefinition, ItemName, Items,
            category::{BaseCategory, Category},
        },
        level_tables::LevelTables,
        packs::{GenerateError, ItemReward, Packs, RewardCollection},
        store_catalogs::{StoreArticleName, StoreCatalogs},
    },
    services::{characters::acquire_item_character, items::get_droppable_items},
};
use chrono::Utc;
use log::debug;
use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, ops::DerefMut};
use thiserror::Error;
use uuid::Uuid;

pub struct ActivityService;

/// Errors that can occur while processing an
/// article purchase
#[derive(Debug, Error)]
pub enum ArticlePurchaseError {
    /// Couldn't find the article requested
    #[error("Unknown article")]
    UnknownArticle,
    /// Server definition error, article associated item was
    /// not present in the item definitions
    #[error("Unknown article item")]
    UnknownArticleItem,
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

impl ActivityService {
    pub async fn process_event(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
    ) -> anyhow::Result<ActivityResult> {
        let mut result = ActivityResult::default();

        Self::process_event_inner(db, user, event, &mut result).await?;

        // Update the current user currencies
        result.currencies = CurrencyRepository::get_by_user(db.deref_mut(), user.id).await?;

        Ok(result)
    }

    pub async fn process_events(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        events: Vec<ActivityEvent>,
    ) -> anyhow::Result<ActivityResult> {
        let mut result = ActivityResult::default();

        for event in events {
            Self::process_event_inner(db, user, event, &mut result).await?;
        }

        // Update the current user currencies
        result.currencies = CurrencyRepository::get_by_user(db.deref_mut(), user.id).await?;

        Ok(result)
    }

    /// Processes the inner portion of an event adding its results
    /// onto an existing result set.
    ///
    /// Doesn't update [ActivityResult::currencies]
    pub async fn process_event_inner(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        debug!("Processing Activity: {:?}", event);

        match event.name {
            ActivityName::ItemConsumed => {
                Self::process_item_consumed(db, user, event, result).await?;
            }
            ActivityName::BadgeEarned => {}
            ActivityName::ArticlePurchased => {
                Self::process_article_purchased(db, user, event, result).await?;
            }
            ActivityName::MissionFinished => {}
            ActivityName::StrikeTeamMissionFinished => {}
            ActivityName::EquipmentUpdated => {}
            ActivityName::EquipmentAttachmentUpdated => {}
            ActivityName::SkillPurchased => {}
            ActivityName::CharacterLevelUp => {}
            ActivityName::PrestigeLevelUp => {}
            ActivityName::PathfinderRatingUpdated => {}
            ActivityName::StrikeTeamRecruited => {}
            ActivityName::Named(_) => {}
        }

        // TODO: Update challenges
        Ok(())
    }

    pub async fn process_article_purchased(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        let catalogs = StoreCatalogs::get();
        let item_definitions = Items::get();
        let classes = Classes::get();
        let level_tables = LevelTables::get();

        let article_name: StoreArticleName = event.attribute_uuid("articleName")?;
        let stack_size: u32 = event.attribute_u32("count")?;

        // Find the article we are looking for
        let article = catalogs
            .catalog
            .get_article(&article_name)
            // Article doesn't exist anymore
            .ok_or(ArticlePurchaseError::UnknownArticle)?;

        // Find the item given by the article
        let item_definition = item_definitions
            .by_name(&article.item_name)
            .ok_or(ArticlePurchaseError::UnknownArticleItem)?;

        // Give the user the article item
        {
            // TODO: Check that the user hasn't already reached the item capacity

            let item = InventoryItemsRepository::add_item(
                db.deref_mut(),
                CreateInventoryItemDto {
                    user_id: user.id,
                    definition_name: item_definition.name,
                    stack_size,
                    capacity: item_definition.capacity,
                    created_at: Utc::now(),
                },
            )
            .await?;

            result.add_item(item, stack_size, item_definition);

            // Handle character creation for character items
            if item_definition.category.base_eq(&BaseCategory::Characters) {
                acquire_item_character(db, user, &item_definition.name, classes, level_tables)
                    .await?;
            }
        }

        Ok(())
    }

    /// Handles granting rewards and other changes from consuming
    /// an inventory item
    pub async fn process_item_consumed(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        let item_definitions = Items::get();
        let classes = Classes::get();
        let level_tables = LevelTables::get();
        let packs = Packs::get();

        let category: Category = event.attribute_parsed("category")?;
        let definition_name: ItemName = event.attribute_uuid("definitionName")?;
        let _count: u32 = event.attribute_u32("count")?;

        let mut rewards: RewardCollection = RewardCollection::default();

        match category.base() {
            BaseCategory::ItemPack => {
                // Find the item pack
                let pack = packs
                    .by_name(&definition_name)
                    .ok_or(ItemConsumeError::PackNotImplemented(definition_name))?;

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

            let item = InventoryItemsRepository::add_item(
                db.deref_mut(),
                CreateInventoryItemDto {
                    user_id: user.id,
                    definition_name,
                    stack_size,
                    capacity: definition.capacity,
                    created_at: Utc::now(),
                },
            )
            .await?;

            result.add_item(item, stack_size, definition);

            // Handle character creation for character items
            if definition.category.base_eq(&BaseCategory::Characters) {
                acquire_item_character(db, user, &definition.name, classes, level_tables).await?;
            }
        }

        Ok(())
    }
}

/// Represents the result produced from processing an [ActivityEvent]
#[derive(Debug, Default, Clone)]
pub struct ActivityResult {
    /// The previous character XP
    pub previous_xp: u32,
    /// The current character XP
    pub current_xp: u32,
    /// The amount of XP gained
    pub gained_xp: u32,

    /// The previous character level
    pub previous_level: u32,
    /// The current character level
    pub current_level: u32,

    /// Present in strike team activity resolves
    pub character_class_name: Option<Uuid>,

    /// The number of challenges completed
    pub challenges_completed: u32,
    /// Challenges that were updates
    pub challenges_updated: Vec<ChallengeUpdated>,

    /// Unknown field
    pub news_triggered: u32,
    /// The current currency amounts that the player has
    pub currencies: Vec<CurrencyDto>,
    /// The different currency amounts that were earned
    pub currency_earned: Vec<CurrencyDto>,

    /// Items that were earned from the activity
    pub items_earned: Vec<InventoryItemDto>,
    /// Definitions for the items from `items_earned`
    pub item_definitions: Vec<&'static ItemDefinition>,

    /// Entitlements that were granted from the activity
    ///
    /// TODO: Haven't encountered a value for this yet so its untyped
    pub entitlements_granted: Vec<Value>,

    /// Prestige progression that resulted from the activity
    pub prestige_progression: PrestigeProgression,
}

impl ActivityResult {
    /// Adds a new item to the result. Updates the `item` stack size to match
    /// the provided `stack_size` to ensure its correct
    pub fn add_item(
        &mut self,
        mut item: InventoryItemDto,
        stack_size: u32,
        definition: &'static ItemDefinition,
    ) {
        item.stack_size = stack_size;

        self.items_earned.push(item);
        self.item_definitions.push(definition);
    }
}

impl Serialize for ActivityResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_struct("ActivityResult", 18)?;
        value.serialize_field("previousXp", &self.previous_xp)?;
        value.serialize_field("xp", &self.current_xp)?;
        value.serialize_field("xpGained", &self.gained_xp)?;

        value.serialize_field("previousLevel", &self.previous_level)?;
        value.serialize_field("level", &self.current_level)?;
        value.serialize_field("levelUp", &(self.current_level != self.previous_level))?;

        if let Some(character_class_name) = &self.character_class_name {
            value.serialize_field("characterClassName", character_class_name)?;
        }

        value.serialize_field("challengesUpdatedCount", &self.challenges_updated.len())?;
        value.serialize_field("challengesCompletedCount", &self.challenges_completed)?;
        value.serialize_field("challengesUpdated", &self.challenges_updated)?;

        // Collect the updated challenge IDs for serialization
        let challenge_ids: Vec<ChallengeId> = self
            .challenges_updated
            .iter()
            .map(|value| value.challenge_id)
            .collect();

        value.serialize_field("updatedChallengeIds", &challenge_ids)?;
        value.serialize_field("newsTriggered", &self.news_triggered)?;
        value.serialize_field("currencies", &self.currencies)?;
        value.serialize_field("currencyEarned", &self.currency_earned)?;
        value.serialize_field("itemsEarned", &self.items_earned)?;
        value.serialize_field("itemDefinitions", &self.item_definitions)?;
        value.serialize_field("entitlementsGranted", &self.entitlements_granted)?;
        value.serialize_field("prestigeProgressionMap", &self.prestige_progression)?;
        value.end()
    }
}

/// Type alias for a [Uuid] representing the name of a prestige level table
pub type PrestigeName = Uuid;

/// Represents the difference between
#[derive(Debug, Clone, Default, Serialize)]
pub struct PrestigeProgression {
    /// The previous prestige data
    pub before: HashMap<PrestigeName, PrestigeData>,
    /// The new prestige data
    pub after: HashMap<PrestigeName, PrestigeData>,
}

/// Prestige data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeData {
    /// The name of the prestige level table
    pub name: PrestigeName,
    /// The prestige current level
    pub level: u32,
    /// The prestige current xp
    pub xp: u32,
}

/// Represents a challenge that was updated
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdated {
    /// The ID of the challenge that was updated
    pub challenge_id: ChallengeId,
    /// Counters that were updated
    pub counters: Vec<ChallengeUpdateCounter>,
    /// The change of status for the challenge update
    pub status_change: ChallengeStatusChange,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeStatusChange {
    /// Notifying the creation of the challenge progress
    Notify,
    /// An existing challenge progress changes
    Changed,
}

/// Represents a challenge counter that was updated
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUpdateCounter {
    /// The name of the counter that was updated
    pub name: ChallengeCounterName,
    /// The new counter value
    pub current_count: u32,
}
