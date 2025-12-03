//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use crate::{
    database::entity::{
        challenge_progress::{ChallengeCounterName, ChallengeId},
        Currency, InventoryItem, User,
    },
    definitions::{
        characters::acquire_item_character,
        classes::Classes,
        items::{BaseCategory, Category, ItemDefinition, ItemName, Items},
        level_tables::LevelTables,
        packs::{GenerateError, ItemReward, Packs, RewardCollection},
        store_catalogs::{StoreArticleName, StoreCatalogs},
    },
};
use log::debug;
use rand::{rngs::StdRng, SeedableRng};
use sea_orm::ConnectionTrait;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};
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
    pub async fn process_event<C>(
        db: &C,
        user: &User,
        event: ActivityEvent,
    ) -> anyhow::Result<ActivityResult>
    where
        C: ConnectionTrait + Send,
    {
        let mut result = ActivityResult::default();

        Self::process_event_inner(db, user, event, &mut result).await?;

        // Update the current user currencies
        result.currencies = Currency::all(db, user).await?;

        Ok(result)
    }

    pub async fn process_events<C>(
        db: &C,
        user: &User,
        events: Vec<ActivityEvent>,
    ) -> anyhow::Result<ActivityResult>
    where
        C: ConnectionTrait + Send,
    {
        let mut result = ActivityResult::default();

        for event in events {
            Self::process_event_inner(db, user, event, &mut result).await?;
        }

        // Update the current user currencies
        result.currencies = Currency::all(db, user).await?;

        Ok(result)
    }

    /// Processes the inner portion of an event adding its results
    /// onto an existing result set.
    ///
    /// Doesn't update [ActivityResult::currencies]
    pub async fn process_event_inner<C>(
        db: &C,
        user: &User,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()>
    where
        C: ConnectionTrait + Send,
    {
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

    pub async fn process_article_purchased<C>(
        db: &C,
        user: &User,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()>
    where
        C: ConnectionTrait + Send,
    {
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

            let item = InventoryItem::add_item(
                db,
                user,
                item_definition.name,
                stack_size,
                item_definition.capacity,
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
    pub async fn process_item_consumed<C>(
        db: &C,
        user: &User,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()>
    where
        C: ConnectionTrait + Send,
    {
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

                // Generate colleciton of rewards
                pack.generate_rewards(db, user, &mut rng, item_definitions, &mut rewards)
                    .await
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

            let item =
                InventoryItem::add_item(db, user, definition.name, stack_size, definition.capacity)
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

/// Represents the name for an activity, contains built in
/// server activity types along with the [Uuid] variant for
/// runtime defined activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityName {
    /// Item was consumed
    ///
    /// Known attributes:
    /// - category (string)
    /// - definitionName (string uuid)
    /// - count (number)
    #[serde(rename = "_itemConsumed")]
    ItemConsumed,
    /// Badge was earned on game completion
    ///
    /// Known attributes:
    /// - badgeName (string)
    /// - count (number)
    #[serde(rename = "_badgeEarned")]
    BadgeEarned,
    /// Article was purchased from the store
    ///
    /// Known attributes:
    /// - currencyName (string)
    /// - articleName (string uuid)
    /// - count (number)
    #[serde(rename = "_articlePurchased")]
    ArticlePurchased,
    /// Mission was finished
    ///
    /// Known attributes:
    /// - percentComplete (number)
    /// - missionTypeName (string uuid)
    /// - count (number)
    #[serde(rename = "_missionFinished")]
    MissionFinished,
    /// Mission was finished by a strike team
    ///
    /// Known attributes:
    /// - success (string boolean)
    /// - count (number)
    #[serde(rename = "_strikeTeamMissionFinished")]
    StrikeTeamMissionFinished,
    /// Equipment was updated
    ///
    /// Known attributes:
    /// - slot (string)
    /// - count (number)
    /// - stackSize (number)
    #[serde(rename = "_equipmentUpdated")]
    EquipmentUpdated,
    /// Equipment attachments were updated
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_equipmentAttachmentUpdated")]
    EquipmentAttachmentUpdated,
    /// Skills were purchased
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_skillPurchased")]
    SkillPurchased,
    /// Character was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - characterClass (string uuid)
    /// - count (number)
    #[serde(rename = "_characterLevelUp")]
    CharacterLevelUp,
    /// Prestige was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - count (number)
    #[serde(rename = "_prestigeLevelUp")]
    PrestigeLevelUp,
    /// Pathfinder rating has changed
    ///
    /// Known attributes
    /// - pathfinderRatingDelta (number)
    #[serde(rename = "_pathfinderRatingUpdated")]
    PathfinderRatingUpdated,
    /// Strike team was recruited
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_strikeTeamRecruited")]
    StrikeTeamRecruited,
    /// Activity represented by a [Uuid] these events can be
    /// published by clients
    #[serde(untagged)]
    Named(Uuid),
}

/// Represents a published activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// The name of the activity event
    pub name: ActivityName,
    /// Data attributes associated with this activity event
    pub attributes: HashMap<AttributeName, ActivityAttribute>,
}

/// Type alias for a string representing an attribute name
pub type AttributeName = String;

/// Represents an attribute within an [ActivityEvent]. These
/// can be numbers or strings
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityAttribute {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Integer(u32),
    /// UUID value
    Uuid(Uuid),
    /// String value
    String(String),
}

impl From<u32> for ActivityAttribute {
    fn from(value: u32) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for ActivityAttribute {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ActivityAttribute {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Uuid> for ActivityAttribute {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<bool> for ActivityAttribute {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl PartialEq for ActivityAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Simple equality
            (Self::Bool(left), Self::Bool(right)) => left.eq(right),
            (Self::Integer(left), Self::Integer(right)) => left.eq(right),
            (Self::String(left), Self::String(right)) => left.eq(right),
            (Self::Uuid(left), Self::Uuid(right)) => left.eq(right),

            // Additional equality for UUID strings (Can be removed once types are strict)
            (Self::Uuid(left), Self::String(right)) => left.to_string().eq(right),
            (Self::String(left), Self::Uuid(right)) => left.eq(&right.to_string()),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct AttributeError {
    /// Name of the attribute
    name: AttributeName,
    /// Cause of the error
    cause: AttributeErrorCause,
}

impl AttributeError {
    fn new(name: &str, cause: AttributeErrorCause) -> Self {
        Self {
            name: name.to_string(),
            cause,
        }
    }
}

#[derive(Debug)]
pub enum AttributeErrorCause {
    /// Attribute was not found
    Missing,
    /// Attribute was an unexpected type
    IncorrectType,
    /// Failed to parse the value
    ParseFailed(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl std::error::Error for AttributeError {}

impl Display for AttributeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error with attribute '{}': {}",
            self.name, self.cause
        ))
    }
}

impl Display for AttributeErrorCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeErrorCause::Missing => f.write_str("Attribute is missing"),
            AttributeErrorCause::IncorrectType => f.write_str("Unexpected attribute type"),
            AttributeErrorCause::ParseFailed(err) => {
                f.write_str("Failed to parse: ")?;
                Display::fmt(err, f)
            }
        }
    }
}

impl ActivityEvent {
    /// Creates a new activity event
    pub fn new(name: ActivityName) -> Self {
        Self {
            name,
            attributes: Default::default(),
        }
    }

    /// Adds an attribute to an activity event
    pub fn with_attribute<V>(mut self, key: &str, value: V) -> Self
    where
        V: Into<ActivityAttribute>,
    {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn attribute_string(&self, key: &str) -> Result<&String, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::String(value) => Ok(value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    /// Obtains an attribute by attempting to parse it
    /// from a [ActivityAttribute::String] value
    pub fn attribute_parsed<V>(&self, key: &str) -> Result<V, AttributeError>
    where
        V: FromStr,
        <V as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        let value = match attribute {
            ActivityAttribute::String(value) => value,
            _ => return Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        };

        value
            .parse()
            // Handle parsing error
            .map_err(|err| {
                AttributeError::new(key, AttributeErrorCause::ParseFailed(Box::new(err)))
            })
    }

    pub fn attribute_uuid(&self, key: &str) -> Result<Uuid, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::Uuid(value) => Ok(*value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    pub fn attribute_u32(&self, key: &str) -> Result<u32, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::Integer(value) => Ok(*value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    /// Obtains the score from the mission activity if it
    /// is present within the attributes
    #[inline]
    pub fn get_score(&self) -> Option<u32> {
        self.attribute_u32("score").ok()
    }

    /// Checks if this activity `attributes` match the provided filter
    pub fn matches_filter(&self, filter: &HashMap<AttributeName, ActivityFilter>) -> bool {
        filter
            .iter()
            // Ensure all attributes match
            .all(|(key, filter)| {
                self.attributes
                    .get(key)
                    // Ensure the value exists and matches
                    .is_some_and(|value| filter.matches(value))
            })
    }
}

/// Describes an activity that can be used to track progress
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDescriptor {
    /// Name of the [ActivityEvent] this descriptor is for
    /// (Can be a [Uuid] or just text such as: "_itemConsumed")
    pub activity_name: ActivityName,
    /// Filtering based on the [ActivityEvent::attributes] for
    /// whether the activity is applicable
    pub filter: HashMap<AttributeName, ActivityFilter>,
    /// The key into [ActivityEvent::attributes] that should be
    /// used for tracking activity progress
    #[serde(rename = "incrementProgressBy")]
    pub progress_key: String,
}

/// Enum for different ways an activity can be filtered against
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityFilter {
    /// Direct value comparison
    Value(ActivityAttribute),
    /// Not equal comparison
    NotEqual {
        /// The value to compare not equal against
        #[serde(rename = "$ne")]
        ne: ActivityAttribute,
    },
}

impl ActivityFilter {
    /// Checks whether the provided [ActivityAttribute] matches this filter
    pub fn matches(&self, other: &ActivityAttribute) -> bool {
        match self {
            Self::Value(value) => value.eq(other),
            Self::NotEqual { ne } => ne.ne(other),
        }
    }
}

impl ActivityDescriptor {
    /// Checks if the provided `activity` matches this descriptor
    pub fn matches(&self, activity: &ActivityEvent) -> bool {
        self.activity_name.eq(&activity.name) && activity.matches_filter(&self.filter)
    }
}

/// Represents the result produced from processing an [ActivityEvent]
#[derive(Debug, Default)]
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
    pub challeges_completed: u32,
    /// Challenges that were updates
    pub challenges_updated: Vec<ChallengeUpdated>,

    /// Unknown field
    pub news_triggered: u32,
    /// The currrent currency amounts that the player has
    pub currencies: Vec<Currency>,
    /// The different currency amounts that were earned
    pub currency_earned: Vec<Currency>,

    /// Items that were earned from the activity
    pub items_earned: Vec<InventoryItem>,
    /// Definitions for the items from `items_earned`
    pub item_definitions: Vec<&'static ItemDefinition>,

    /// Entitlements that were granted from the activity
    ///
    /// TODO: Haven't encounted a value for this yet so its untyped
    pub entitlements_granted: Vec<Value>,

    /// Prestige progression that resulted from the activity
    pub prestige_progression: PrestigeProgression,
}

impl ActivityResult {
    /// Adds a new item to the result. Updates the `item` stack size to match
    /// the provided `stack_size` to ensure its correct
    pub fn add_item(
        &mut self,
        mut item: InventoryItem,
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
        value.serialize_field("challengesCompletedCount", &self.challeges_completed)?;
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
