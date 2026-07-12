//! Pack generation scripts, used to generate the lists of weighted
//! items for randomly generated packs.
//!
//! The randomness used for these packs are only guesses and may not
//! be accurate to the actual game loot tables.

use crate::definitions::items::{ItemDefinition, ItemName};
use anyhow::Context;
use log::debug;
use rand::{distributions::WeightedError, rngs::StdRng, seq::SliceRandom};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};
use thiserror::Error;

use builder::PackBuilder;
use filter::{Filter, Weight};

mod builder;
mod filter;
mod parser;

/// Pack definitions
const PACK_DEFINITIONS: &str = include_str!("../../resources/data/packs.json");

/// Collection of defined [Pack]s
pub struct Packs {
    /// Lookup for packs by [ItemName]
    packs: HashMap<ItemName, Pack>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<Packs> = OnceLock::new();

impl Packs {
    /// Gets a static reference to the global [ChallengeDefinitions] collection
    pub fn get() -> &'static Packs {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let values: HashMap<uuid::Uuid, Pack> =
            serde_json::from_str(PACK_DEFINITIONS).context("Failed to load pack definitions")?;

        debug!("Loaded {} pack definition(s)", values.len());
        Ok(Self { packs: values })
    }

    pub fn by_name(&self, name: &ItemName) -> Option<&Pack> {
        self.packs.get(name)
    }
}

/// Represents a pack that can be used to generate items
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pack {
    /// The name of the pack item
    pub name: ItemName,

    /// Description of the pack / in game name
    pub description: String,

    /// Description of the pack contents (For internal reference)
    contents_description: String,

    /// The collection of item reward this pack provides
    collections: Box<[PackCollection]>,
}

impl Pack {
    /// Creates a new pack builder using the provided name
    #[inline]
    fn builder(
        name: ItemName,
        description: impl Into<String>,
        contents_description: impl Into<String>,
    ) -> PackBuilder {
        PackBuilder::new(name, description.into(), contents_description.into())
    }

    /// Generates a [RewardCollection] from this [Pack] using the provided
    /// random number generator `rng`
    pub fn generate_rewards<'def>(
        &self,
        rng: &mut StdRng,
        droppable_items: &[&'def ItemDefinition],
        rewards: &mut RewardCollection<'def>,
    ) -> Result<(), GenerateError> {
        // Generate rewards from each collection
        for collection in self.collections.iter() {
            collection.generate_rewards(rng, droppable_items, rewards)?;
        }

        Ok(())
    }
}

/// Chance for gaining an item from a specific filter
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PackCollection {
    /// The filter for choosing these pack items
    filter: Filter,
    /// The stack size of each item produced from this collection
    stack_size: u32,
    /// The amount of items to produce from the collection
    /// if [None] they should be given one of every item
    amount: Option<u32>,
}

impl PackCollection {
    /// Creates a new pack item from a filter
    fn new(filter: Filter) -> Self {
        Self {
            filter,
            stack_size: 1,
            amount: Some(1),
        }
    }

    /// Shorthand for specifying a pack item for
    /// a specific item directly by `name`
    #[inline]
    fn named(name: ItemName) -> Self {
        Self::new(Filter::Named(name))
    }

    /// Update the amount of items to produce
    fn amount(mut self, amount: u32) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Update the stack size to produce
    fn stack_size(mut self, stack_size: u32) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Tells the pack item to provide *all* the items that
    /// match the filter rather than a specific amount
    fn all(mut self) -> Self {
        self.amount = None;
        self
    }

    fn generate_rewards<'def>(
        &self,
        rng: &mut StdRng,
        items: &[&'def ItemDefinition],
        rewards: &mut RewardCollection<'def>,
    ) -> Result<(), GenerateError> {
        // Collection of items with the filter and weights applied
        let weighted_items: Vec<(&ItemDefinition, Weight)> = items
            .iter()
            .filter_map(|item| {
                let weight = self.filter.apply_filter(item)?;
                // Ensure non zero weights
                let weight = weight.max(1);

                Some((*item, weight))
            })
            .collect();

        // Handle complete collection rewards
        let amount = match self.amount {
            Some(value) => value,
            None => {
                // Add all the matching items
                weighted_items
                    .into_iter()
                    .for_each(|(definition, _)| rewards.add_reward(definition, self.stack_size));

                return Ok(());
            }
        };

        // There was no applicable items
        if weighted_items.is_empty() {
            return Ok(());
        }

        // Sample random items from the collection
        weighted_items
            .choose_multiple_weighted(rng, amount as usize, |value| value.1)?
            // Add the reward
            .for_each(|(definition, _)| rewards.add_reward(definition, self.stack_size));

        Ok(())
    }
}

/// Error generating pack rewards
#[derive(Debug, Error)]
pub enum GenerateError {
    /// Failed to do weighted randomness
    #[error(transparent)]
    Weight(#[from] WeightedError),
    /// Failed to query the database for item ownership
    #[error("Server error")]
    Database(#[from] DbErr),
}

/// Wrapper around a collection of rewards to make adding
/// new rewards without duplicates easier
#[derive(Default)]
pub struct RewardCollection<'a> {
    pub rewards: Vec<ItemReward<'a>>,
}

/// Represents an awarded item along with the amount of the item
/// that was rewarded
pub struct ItemReward<'a> {
    pub definition: &'a ItemDefinition,
    pub stack_size: u32,
}

impl<'a> RewardCollection<'a> {
    fn add_reward(&mut self, definition: &'a ItemDefinition, stack_size: u32) {
        let existing = self
            .rewards
            .iter_mut()
            .find(|value| value.definition.name.eq(&definition.name));

        // Increase stack size for existing items
        if let Some(existing) = existing {
            existing.stack_size += stack_size;
        } else {
            self.rewards.push(ItemReward {
                definition,
                stack_size,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::Packs;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = Packs::load().unwrap();
    }
}
