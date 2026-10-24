use pocket_ark_database::dto::{
    challenge_progress::ChallengeId, currency::CurrencyDto, inventory_items::InventoryItemDto,
};
use pocket_ark_definitions::items::ItemDefinition;
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::services::activity::result::{ActivityResult, ChallengeUpdated, PrestigeProgression};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpActivityResult {
    previous_xp: u32,
    xp: u32,
    xp_gained: u32,

    previous_level: u32,
    level: u32,
    level_up: bool,

    character_class_name: Option<Uuid>,

    challenges_updated_count: usize,
    challenges_completed_count: usize,
    challenges_updated: Vec<ChallengeUpdated>,
    updated_challenge_ids: Vec<ChallengeId>,

    news_triggered: u32,

    currencies: Vec<CurrencyDto>,
    currency_earned: Vec<CurrencyDto>,

    items_earned: Vec<InventoryItemDto>,
    item_definitions: Vec<&'static ItemDefinition>,

    entitlements_granted: Vec<Value>,

    prestige_progression_map: PrestigeProgression,
}

impl From<ActivityResult> for HttpActivityResult {
    fn from(value: ActivityResult) -> Self {
        let updated_challenge_ids = value
            .challenges_updated
            .iter()
            .map(|value| value.challenge_id)
            .collect();

        Self {
            previous_xp: value.previous_xp,
            xp: value.current_xp,
            xp_gained: value.gained_xp,
            previous_level: value.previous_level,
            level: value.current_level,
            level_up: value.current_level != value.previous_level,
            character_class_name: value.character_class_name,
            challenges_updated_count: value.challenges_updated.len(),
            challenges_completed_count: value.challenges_completed as usize,
            challenges_updated: value.challenges_updated,
            updated_challenge_ids,
            news_triggered: value.news_triggered,
            currencies: value.currencies,
            currency_earned: value.currency_earned,
            items_earned: value.items_earned,
            item_definitions: value.item_definitions,
            entitlements_granted: value.entitlements_granted,
            prestige_progression_map: value.prestige_progression,
        }
    }
}
