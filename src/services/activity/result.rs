use pocket_ark_database::dto::{
    challenge_progress::{ChallengeCounterName, ChallengeId},
    currency::CurrencyDto,
    inventory_items::InventoryItemDto,
};
use pocket_ark_definitions::items::ItemDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug};
use uuid::Uuid;

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
