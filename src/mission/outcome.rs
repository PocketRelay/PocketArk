use uuid::Uuid;

use crate::{
    database::dto::{
        inventory_items::InventoryItemDto,
        shared_data::{SharedDataDto, SharedProgression},
    },
    definitions::{
        challenges::{ChallengeCounter, ChallengeDefinition},
        currency::CurrencyType,
        level_tables::ProgressionXp,
    },
    http::models::mission::{PlayerInfoBadge, RewardSource},
    services::activity::{PrestigeData, PrestigeProgression},
};
use std::collections::HashMap;

/// Temporary data for storing changes to challenges
pub struct ChallengeProgressChange {
    /// The challenge definition
    pub definition: &'static ChallengeDefinition,
    /// The counter to change
    pub counter: &'static ChallengeCounter,
    /// The progress made to the challenge
    pub progress: u32,
}

#[derive(Default)]
pub struct MissionOutcomeDataBuilder {
    pub score: u32,
    pub xp_earned: u32,
    pub reward_sources: Vec<RewardSource>,
    pub total_currency: HashMap<CurrencyType, u32>,
    pub prestige_progression: PrestigeProgression,
    pub items_earned: Vec<InventoryItemDto>,
    pub challenges_updates: Vec<ChallengeProgressChange>,
    pub badges: Vec<PlayerInfoBadge>,
    pub leveling: Option<MissionOutcomeLevelingChange>,
    pub shared_progression: Vec<SharedProgression>,
}

pub struct MissionOutcomeLevelingChange {
    pub xp: ProgressionXp,
    pub level: u32,
}

impl MissionOutcomeDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn append_prestige(map: &mut HashMap<Uuid, PrestigeData>, progression: &[SharedProgression]) {
        // Insert the before change
        progression.iter().for_each(|value| {
            map.insert(
                value.name,
                PrestigeData {
                    level: value.level,
                    name: value.name,
                    xp: value.xp.current,
                },
            );
        });
    }

    pub fn append_prestige_before(&mut self, progression: &[SharedProgression]) {
        Self::append_prestige(&mut self.prestige_progression.before, progression)
    }

    pub fn append_prestige_after(&mut self, progression: &[SharedProgression]) {
        Self::append_prestige(&mut self.prestige_progression.after, progression)
    }

    pub fn add_challenge_progress(&mut self, update: ChallengeProgressChange) {
        let existing = self
            .challenges_updates
            .iter_mut()
            // Check if theres already a matching progress update
            .find(|value| {
                value.definition.base.name == update.definition.base.name
                    && value.counter.name == update.counter.name
            });

        if let Some(existing) = existing {
            existing.progress = existing.progress.saturating_add(update.progress);
        } else {
            self.challenges_updates.push(update);
        }
    }

    pub fn add_reward_xp(&mut self, name: &str, xp: u32) {
        // Append earned xp
        self.xp_earned = self.xp_earned.saturating_add(xp);

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            existing.xp = existing.xp.saturating_add(xp);
        } else {
            self.reward_sources.push(RewardSource {
                currencies: HashMap::new(),
                xp,
                name: name.to_string(),
            });
        }
    }

    pub fn add_reward_currency(&mut self, name: &str, currency: CurrencyType, value: u32) {
        // Append currencies to total currency

        if let Some(existing) = self.total_currency.get_mut(&currency) {
            *existing += value
        } else {
            self.total_currency.insert(currency, value);
        }

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            // Update currency within reward
            if let Some(existing) = existing.currencies.get_mut(&currency) {
                *existing = existing.saturating_add(value);
            } else {
                existing.currencies.insert(currency, value);
            }
        } else {
            let mut currencies = HashMap::new();
            currencies.insert(currency, value);

            self.reward_sources.push(RewardSource {
                currencies,
                xp: 0,
                name: name.to_string(),
            });
        }
    }
}
