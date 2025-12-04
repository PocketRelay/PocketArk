//! Pack generation scripts, used to generate the lists of weighted
//! items for randomly generated packs.
//!
//! The randomness used for these packs are only guesses and may not
//! be accurate to the actual game loot tables.

use crate::{
    database::entity::{InventoryItem, User},
    definitions::items::{BaseCategory, Category, ItemDefinition, ItemName, ItemRarity, Items},
};
use rand::{distributions::WeightedError, rngs::StdRng, seq::SliceRandom};
use sea_orm::{ConnectionTrait, DbErr};
use serde_json::Value;
use std::{collections::HashMap, sync::OnceLock};
use thiserror::Error;
use uuid::uuid;

/// Collection of defined [Pack]s
pub struct Packs {
    /// Lookup for packs by [ItemName]
    packs: HashMap<ItemName, Pack>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<Packs> = OnceLock::new();

impl Default for Packs {
    fn default() -> Self {
        Self::new()
    }
}

impl Packs {
    /// Gets a static reference to the global [ChallengeDefinitions] collection
    pub fn get() -> &'static Packs {
        STORE.get_or_init(Self::new)
    }

    fn new() -> Self {
        Self {
            packs: generate_packs(),
        }
    }

    pub fn by_name(&self, name: &ItemName) -> Option<&Pack> {
        self.packs.get(name)
    }
}

/// Builder for creating [Pack]s
struct PackBuilder {
    /// The name of the pack item
    name: ItemName,
    /// The collection of item reward this pack provides
    collections: Vec<PackCollection>,
}

impl PackBuilder {
    /// Creates a new pack builder using the provided name
    fn new(name: ItemName) -> Self {
        Self {
            name,
            collections: Vec::new(),
        }
    }

    /// Adds a new collection to the pack
    fn add(mut self, chance: PackCollection) -> Self {
        self.collections.push(chance);
        self
    }

    /// Builds the finished [Pack]
    fn build(self) -> Pack {
        Pack {
            name: self.name,
            collections: self.collections.into_boxed_slice(),
        }
    }
}

/// Represents a pack that can be used to generate items
pub struct Pack {
    /// The name of the pack item
    pub name: ItemName,
    /// The collection of item reward this pack provides
    collections: Box<[PackCollection]>,
}

impl Pack {
    /// Creates a new pack builder using the provided name
    #[inline]
    fn builder(name: ItemName) -> PackBuilder {
        PackBuilder::new(name)
    }

    /// Generates a [RewardCollection] from this [Pack] using the provided
    /// random number generator `rng`
    ///
    /// Requires database access for checking item ownership requirement
    /// in order to match
    ///
    pub async fn generate_rewards<'def, C>(
        &self,
        db: &C,
        user: &User,
        rng: &mut StdRng,
        defs: &'def Items,
        rewards: &mut RewardCollection<'def>,
    ) -> Result<(), GenerateError>
    where
        C: ConnectionTrait + Send,
    {
        // Creates a list of items that are applicable for dropping (If they match filters)
        // this step is done so unlock definitions and droppability don't have to be
        // done for every single collection filter
        let mut items: Vec<&ItemDefinition> = defs
            // Iterate all the definitions
            .all()
            .iter()
            // Only include droppable items
            .filter(|item| item.is_droppable())
            .collect();

        // Collection of requirements for items (For requirement filtering)
        let required_items: Vec<ItemName> = items
            .iter()
            .filter_map(|item| item.unlock_definition.as_ref())
            .copied()
            .collect();

        // Collect the owned items
        let owned_items: Vec<InventoryItem> =
            InventoryItem::all_by_names(db, user, required_items).await?;

        // Remove items that don't meet the owned requirement
        items.retain(|definition| {
            let unlock_def_name: &ItemName = match definition.unlock_definition.as_ref() {
                Some(value) => value,
                // No unlocking requirement
                None => return true,
            };

            // Find the item definition for the lock requirment
            let unlock_def: &ItemDefinition = match defs.by_name(unlock_def_name) {
                Some(value) => value,
                // Unlock definition doesn't exist (Filter out, item must be invalid)
                None => return false,
            };

            // Ensure the user owns atleast one of the item
            let owned_item = match owned_items
                .iter()
                .find(|item| item.definition_name == definition.name)
            {
                Some(value) => value,

                // Requirement of owning the item not met
                None => return false,
            };

            // If there is a max capacity for the item ensure its been reached
            unlock_def
                .capacity
                .is_some_and(|capacity| owned_item.stack_size == capacity)
        });

        // Generate rewards from each collection
        for collection in self.collections.iter() {
            collection.generate_rewards(rng, &items, rewards)?;
        }

        Ok(())
    }
}

/// Chance for gaining an item from a specific filter
#[derive(Debug)]
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

/// Type used for the weight of a filter result
type Weight = u32;

/// Item filtering
#[derive(Debug, Clone)]
enum Filter {
    /// Filter that never matches anything (Fallback)
    Never,

    /// Specific item referenced by [ItemName]
    Named(ItemName),
    /// Require the item to be a specific rarity
    Rarity(ItemRarity),
    /// Item from a selection of a category
    Category(Category),
    /// Filter based on a specific item attribute
    Attribute(String, Value),

    /// Filter matching many filters. Only one of the filters needs to
    /// pass, will compare all the filters and the weight will become
    /// the sum of all matching filters
    Many(Vec<Filter>),
    /// Filter matching only when both filters match
    And(Box<Filter>, Box<Filter>),
    /// Filter matching when either filter matches
    Or(Box<Filter>, Box<Filter>),
    /// Filter requiring the other filter does not match
    Not(Box<Filter>),

    /// Filter with an additional weighted randomness amount
    Weighted(Box<Filter>, Weight),
}

#[allow(unused)]
impl Filter {
    /// Creates a new filter matching all of the provided filters
    fn all<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Filter>,
    {
        filters
            .into_iter()
            .reduce(|accum, value| accum.and(value))
            .unwrap_or(Filter::Never)
    }

    /// Creates a new filter matching any of the provided filters
    fn any<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Filter>,
    {
        filters
            .into_iter()
            .reduce(|accum, value| accum.or(value))
            .unwrap_or(Filter::Never)
    }

    /// Creates a filter that matches all the provided `rarities`
    fn rarities<I>(rarities: I) -> Self
    where
        I: IntoIterator<Item = ItemRarity>,
    {
        Self::any(rarities.into_iter().map(Self::Rarity))
    }

    /// Filter that accepts any rarity
    fn any_rarity() -> Self {
        Self::rarities([
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::UltraRare,
        ])
    }

    /// Creates a filter that matches all the provided `categories`
    fn categories<I>(categories: I) -> Self
    where
        I: IntoIterator<Item = Category>,
    {
        Self::any(categories.into_iter().map(Self::Category))
    }

    #[inline]
    const fn base_category(category: BaseCategory) -> Self {
        Self::Category(Category::Base(category))
    }

    /// Creates a filter that matches all the provided `base_categories`
    fn base_categories<I>(categories: I) -> Self
    where
        I: IntoIterator<Item = BaseCategory>,
    {
        Self::any(categories.into_iter().map(Self::base_category))
    }

    /// Creates an attribute filter from the provided key and value
    fn attribute<K, V>(key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        Self::Attribute(key.into(), value.into())
    }

    /// Creates an attributes filter from an iterator of key
    /// value pairs requires all the attribute match
    fn attributes<I, K, V>(attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        Self::all(
            attributes
                .into_iter()
                .map(|(key, value)| Self::attribute(key, value)),
        )
    }

    /// Combines the current filter with another filter using
    /// AND logic
    fn and(self, other: Self) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    /// Combines the current filter with another filter using
    /// OR logic
    fn or(self, other: Self) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    /// Inverts the current filter
    fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Applies a weight to the filter
    fn weight(self, weight: u32) -> Self {
        Self::Weighted(Box::new(self), weight)
    }

    /// Creates a new [Filter::Many] filter from an iterator
    /// of filters
    fn many<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Many(iter.into_iter().collect())
    }

    /// Combines the two filters, used to merge additional weights
    #[inline]
    fn merge(self, filter: Filter) -> Self {
        Self::many([self, filter])
    }

    /// Combines the many filters, used to merge additional weights
    #[inline]
    fn merge_many<I>(self, filters: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::many(Some(self).into_iter().chain(filters))
    }

    /// Applies the filter against the provided `item` definition
    /// returns [None] if the value did not match otherwise returns
    /// [Some] with the calculated [FilterWeight]
    fn apply_filter(&self, item: &ItemDefinition) -> Option<Weight> {
        match self {
            Filter::Named(name) => {
                if name != &item.name {
                    return None;
                }

                Some(0)
            }
            Filter::Rarity(rarity) => {
                let item_rarity = item.rarity.as_ref()?;
                if rarity != item_rarity {
                    return None;
                }

                Some(rarity.weight())
            }
            Filter::Category(category) => {
                let item_category = &item.category;

                if item_category.is_within(category) {
                    return None;
                }

                Some(0)
            }
            Filter::Attribute(key, value) => {
                let matches = item
                    .custom_attributes
                    .get(key)
                    .is_some_and(|attr| attr.eq(value));

                if matches { Some(0) } else { None }
            }
            Filter::Many(filters) => {
                let mut weight_sum = 0;
                let mut matches = false;

                for filter in filters {
                    if let Some(weight) = filter.apply_filter(item) {
                        weight_sum += weight;
                        matches = true;
                    }
                }

                if matches { Some(weight_sum) } else { None }
            }
            Filter::And(left, right) => {
                let left = left.apply_filter(item)?;
                let right = right.apply_filter(item)?;
                Some(left + right)
            }
            Filter::Or(left, right) => {
                if let Some(left) = left.apply_filter(item) {
                    Some(left)
                } else {
                    right.apply_filter(item)
                }
            }
            Filter::Not(filter) => {
                if filter.apply_filter(item).is_some() {
                    None
                } else {
                    Some(0)
                }
            }
            Filter::Weighted(filter, weight) => filter
                .apply_filter(item)
                // Add the additional weight
                .map(|filter_weight| filter_weight + *weight),
            Filter::Never => None,
        }
    }
}

/// Generates the collection of packs to use
fn generate_packs() -> HashMap<ItemName, Pack> {
    // Category filter based on normal items
    let items_filter = Filter::base_categories([
        BaseCategory::Weapons,
        BaseCategory::WeaponMods,
        BaseCategory::Boosters,
        BaseCategory::Consumable,
        BaseCategory::Equipment,
        BaseCategory::WeaponsSpecialized,
        BaseCategory::WeaponModsEnhanced,
    ]);

    // Item filter extended to include characters
    let items_and_characters_filter = items_filter
        .clone()
        .or(Filter::base_category(BaseCategory::Characters));

    // "Includes the Cobra RPG, First Aid Pack, Ammo Pack, and Revive Pack, as well as a Random Booster."
    let supply_pack = Pack::builder(uuid!("c5b3d9e6-7932-4579-ba8a-fd469ed43fda"))
        // COBRA RPG
        .add(PackCollection::named(uuid!(
            "eaefec2a-d892-498b-a175-e5d2048ae39a"
        )))
        // REVIVE PACK
        .add(PackCollection::named(uuid!(
            "af39be6b-0542-4997-b524-227aa41ae2eb"
        )))
        // AMMO PACK
        .add(PackCollection::named(uuid!(
            "2cc0d932-8e9d-48a6-a6e8-a5665b77e835"
        )))
        // FIRST AID PACK
        .add(PackCollection::named(uuid!(
            "4d790010-1a79-4bd0-a79b-d52cac068a3a"
        )))
        // Random Boosters
        .add(PackCollection::new(Filter::Category(Category::Base(
            BaseCategory::Boosters,
        ))))
        .build();

    // "Contains 5 random items or characters, with a small chance that 1 will be Uncommon"
    let basic_pack = Pack::builder(uuid!("c6d431eb-325f-4765-ab8f-e48d7b58aa36"))
        // 4 common items/characters
        .add(
            PackCollection::new(
                Filter::Rarity(ItemRarity::Common)
                    // That are normal items or characters
                    .and(items_and_characters_filter.clone()),
            )
            .amount(4),
        )
        // 1 item/character that is uncommon or common
        .add(PackCollection::new(
            // 8:1 chance for getting common over uncommon
            Filter::rarities([ItemRarity::Common, ItemRarity::Uncommon])
                .and(items_and_characters_filter.clone()),
        ))
        .build();

    // "Includes 5 each of the Cobra RPG, First Aid Pack, Ammo Pack, and Revive Pack, as well as 5 Random Boosters."
    let jumbo_supply_pack = Pack::builder(uuid!("e4f4d32a-90c3-4f5c-9362-3bb5933706c7"))
        // 5x COBRA RPG
        .add(PackCollection::named(uuid!("eaefec2a-d892-498b-a175-e5d2048ae39a")).stack_size(5))
        // 5x REVIVE PACK
        .add(PackCollection::named(uuid!("af39be6b-0542-4997-b524-227aa41ae2eb")).stack_size(5))
        // 5x AMMO PACK
        .add(PackCollection::named(uuid!("2cc0d932-8e9d-48a6-a6e8-a5665b77e835")).stack_size(5))
        // 5x FIRST AID PACK
        .add(PackCollection::named(uuid!("4d790010-1a79-4bd0-a79b-d52cac068a3a")).stack_size(5))
        // 5 Random Boosters
        .add(
            PackCollection::new(Filter::Category(Category::Base(BaseCategory::Boosters))).amount(5),
        )
        .build();

    // "Contains 2 of each Uncommon ammo booster, plus 2 additional boosters, at least 1 of which is Rare or better."
    let ammo_priming_pack = Pack::builder(uuid!("eddfd7b7-3476-4ad7-9302-5cfe77ee4ea6"))
        .add(
            PackCollection::new(
                // Uncommon ammo booster
                Filter::Category(Category::Base(BaseCategory::Boosters))
                    .and(Filter::attributes([("consumableType", "Ammo")]))
                    .and(Filter::Rarity(ItemRarity::Uncommon)),
            )
            // Give them all the uncommon ammo boosters
            .all()
            .stack_size(2),
        )
        // First booster (Can be any rarity)
        .add(PackCollection::new(Filter::Category(Category::Base(
            BaseCategory::Boosters,
        ))))
        // Second booster (Must be rare or better)
        .add(PackCollection::new(
            Filter::Category(Category::Base(BaseCategory::Boosters))
                .and(Filter::rarities([ItemRarity::Rare, ItemRarity::UltraRare])),
        ))
        .build();

    // "Contains 5 random consumables or weapon mods, including at least 1 Uncommon, with a small chance for a Rare."
    let technical_mods_pack = Pack::builder(uuid!("975f87f5-0242-4c73-9e0f-6e4033b22ee9"))
        .add(
            PackCollection::new(
                Filter::base_categories([
                    BaseCategory::Consumable,
                    BaseCategory::WeaponMods,
                    BaseCategory::WeaponModsEnhanced,
                ])
                .and(Filter::Rarity(ItemRarity::Common)),
            )
            .amount(4),
        )
        .add(PackCollection::new(
            Filter::base_categories([
                BaseCategory::Consumable,
                BaseCategory::WeaponMods,
                BaseCategory::WeaponModsEnhanced,
            ])
            .and(Filter::rarities([ItemRarity::Uncommon, ItemRarity::Rare])),
        ))
        .build();

    // "Contains 5 random items or characters, including at least 1 Uncommon, with a small chance for a Rare"
    let advanced_pack = Pack::builder(uuid!("974a8c8e-08bc-4fdb-bede-43337c255df8"))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    .and(Filter::Rarity(ItemRarity::Common)),
            )
            .amount(4),
        )
        .add(PackCollection::new(
            items_and_characters_filter
                .clone()
                .and(Filter::rarities([ItemRarity::Uncommon, ItemRarity::Rare])),
        ))
        .build();

    // "Contains 5 random items or characters, including at least 1 Rare, with a small chance for an Ultra-Rare "
    let expert_pack = Pack::builder(uuid!("b6fe6a9f-de70-463a-bcc5-a1b146067470"))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    .and(Filter::rarities([ItemRarity::Common, ItemRarity::Uncommon])),
            )
            .amount(4),
        )
        .add(PackCollection::new(
            items_and_characters_filter
                .clone()
                .and(Filter::rarities([ItemRarity::Uncommon, ItemRarity::Rare])),
        ))
        .build();

    // "Contains 5 random items or characters, including at least 2 that are Rare or better, with a higher chance for characters."
    let reserves_pack = Pack::builder(uuid!("731b16c9-3a97-4166-a2f7-e79c8b45128a"))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Apply additional weight to characters
                    .merge(Filter::base_category(BaseCategory::Characters).weight(32))
                    // Exculde Rare and Ultra rare from this selection
                    .and(Filter::rarities([ItemRarity::Rare, ItemRarity::UltraRare]).not()),
            )
            .amount(3),
        )
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Apply additional weight to characters
                    .merge(Filter::base_category(BaseCategory::Characters).weight(32))
                    // Only Rare and Ultra rare
                    .and(Filter::rarities([ItemRarity::Rare, ItemRarity::UltraRare])),
            )
            .amount(2),
        )
        .build();

    // "Contains 5 random items or characters, including at least 2 that are Rare or better, with a higher chance for weapons."
    let arsenal_pack = Pack::builder(uuid!("29c47d42-5830-435b-943f-bf6cf04145e1"))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Apply additional weight to weapons
                    .merge(
                        Filter::any([
                            Filter::base_category(BaseCategory::Weapons),
                            Filter::base_category(BaseCategory::WeaponsSpecialized),
                        ])
                        .weight(32),
                    )
                    // Exculde Rare and Ultra rare from this selection
                    .and(Filter::rarities([ItemRarity::Rare, ItemRarity::UltraRare]).not()),
            )
            .amount(3),
        )
        .add(
            PackCollection::new(
                // Items or characters weighted on weapons
                items_and_characters_filter
                    .clone()
                    // Apply additional weight to weapons
                    .merge(
                        Filter::any([
                            Filter::base_category(BaseCategory::Weapons),
                            Filter::base_category(BaseCategory::WeaponsSpecialized),
                        ])
                        .weight(32),
                    )
                    // Only Rare and Ultra rare
                    .and(Filter::rarities([ItemRarity::Rare, ItemRarity::UltraRare])),
            )
            .amount(2),
        )
        .build();

    // "Contains 5 random items or characters, including at least 2 that are Rare, with a higher chance for at least 1 Ultra-Rare"
    let premium_pack = Pack::builder(uuid!("8344cd62-2aed-468d-b155-6ae01f1f2405"))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Add increased chance for ultra rare
                    .merge(Filter::Rarity(ItemRarity::UltraRare).weight(8)),
            )
            .amount(3),
        )
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    .and(Filter::Rarity(ItemRarity::Rare)),
            )
            .amount(2),
        )
        .build();

    // "Contains 25 random items or characters, including at least 10 that are Rare, with 5 improved chances for an Ultra-Rare."
    let jumbo_premium_pack = Pack::builder(uuid!("e3e56e89-b995-475f-8e75-84bf27dc8297"))
        .add(PackCollection::new(items_and_characters_filter.clone()).amount(10))
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Add increased chance for ultra rare
                    .merge(Filter::Rarity(ItemRarity::UltraRare).weight(8)),
            )
            .amount(5),
        )
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    .and(Filter::Rarity(ItemRarity::Rare)),
            )
            .amount(10),
        )
        .build();

    // "Contains 5 random items or characters, including at least 1 Uncommon, with a small chance for a Rare"
    let bonus_reward_pack = |name: ItemName| {
        Pack::builder(name)
            .add(PackCollection::new(items_and_characters_filter.clone()).amount(4))
            .add(
                PackCollection::new(
                    items_and_characters_filter
                        .clone()
                        .and(Filter::Rarity(ItemRarity::Uncommon)),
                )
                .amount(1),
            )
            .build()
    };

    let random_mod_pack = |name: ItemName, rarity: ItemRarity| -> Pack {
        Pack::builder(name)
            .add(PackCollection::new(
                Filter::base_categories([
                    BaseCategory::WeaponMods,
                    BaseCategory::WeaponModsEnhanced,
                ])
                .and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    let random_weapon_pack = |name: ItemName, rarity: ItemRarity| -> Pack {
        Pack::builder(name)
            .add(PackCollection::new(
                Filter::base_categories([BaseCategory::Weapons, BaseCategory::WeaponsSpecialized])
                    .and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    let random_character_pack = |name: ItemName, rarity: ItemRarity| -> Pack {
        Pack::builder(name)
            .add(PackCollection::new(
                Filter::base_category(BaseCategory::Characters).and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    // Pack containing a single item
    let item_pack = |name: ItemName, item: ItemName| {
        Pack::builder(name).add(PackCollection::named(item)).build()
    };

    // Marker for a pack that is not yet implemented
    let todo = |name: ItemName| Pack::builder(name).build();

    // List of all the packs
    [
        supply_pack,
        basic_pack,
        jumbo_supply_pack,
        ammo_priming_pack,
        technical_mods_pack,
        advanced_pack,
        expert_pack,
        reserves_pack,
        arsenal_pack,
        premium_pack,
        jumbo_premium_pack,
        bonus_reward_pack(uuid!("cf9cd252-e1f2-4574-973d-d66cd81558d3")),
        bonus_reward_pack(uuid!("ab939baf-3cc0-46a8-8983-5c8e92754a25")),
        // Random mods
        random_mod_pack(
            uuid!("890b2aa6-191f-4162-ae79-a78d23e3c505"),
            ItemRarity::Common,
        ),
        random_mod_pack(
            uuid!("44da78e5-8ceb-4684-983e-794329d4a631"),
            ItemRarity::Uncommon,
        ),
        random_mod_pack(
            uuid!("b104645c-ff63-4081-a3c2-669718d7e570"),
            ItemRarity::Rare,
        ),
        // Random weapons
        random_weapon_pack(
            uuid!("20a2212b-ac19-436f-93c9-143463a813e9"),
            ItemRarity::Uncommon,
        ),
        random_weapon_pack(
            uuid!("aea28dd4-b5be-4994-80ec-825e2b024d4d"),
            ItemRarity::Rare,
        ),
        random_weapon_pack(
            uuid!("e9bfb771-5244-4f33-b318-dd49d79c7edf"),
            ItemRarity::UltraRare,
        ),
        // Random characters
        random_character_pack(
            uuid!("e71d0c00-44f2-4087-a7f7-7a138fbee0e9"),
            ItemRarity::Uncommon,
        ),
        random_character_pack(
            uuid!("53c8b4d7-18bf-4fc3-97cd-2a8366140b0a"),
            ItemRarity::Rare,
        ),
        random_character_pack(
            uuid!("dad9ad62-1f36-4e38-9634-2eda92a83096"),
            ItemRarity::UltraRare,
        ),
        // Single item packs

        // COBRA RPG
        item_pack(
            uuid!("ff6affa2-226b-4c8b-8013-7e7e94335e88"),
            uuid!("eaefec2a-d892-498b-a175-e5d2048ae39a"),
        ),
        // REVIVE PACK
        item_pack(
            uuid!("784e1293-4480-4abd-965e-2c6584f550c8"),
            uuid!("af39be6b-0542-4997-b524-227aa41ae2eb"),
        ),
        // AMMO PACK
        item_pack(
            uuid!("16cdf51b-443a-48e2-ad07-413a3f4370e7"),
            uuid!("2cc0d932-8e9d-48a6-a6e8-a5665b77e835"),
        ),
        // CHARACTER RESPEC
        item_pack(
            uuid!("bc012022-2d42-48d1-88fa-2d905d83d4fd"),
            uuid!("52a2e172-2ae6-49f4-9914-bf3094f3a363"),
        ),
        // EXPERIENCE ENHANCER III
        item_pack(
            uuid!("3a7a1d97-ddb7-4954-85e8-b280c2b9b2dc"),
            uuid!("83d69f5b-3f97-4d41-ad76-99ea37a35ba8"),
        ),
        // EXPERIENCE ENHANCER II
        item_pack(
            uuid!("a26534c9-636c-4022-8d7e-3f76af5fde02"),
            uuid!("4f46229e-51cd-4ece-9a21-731133348088"),
        ),
        // FIRST AID PACK
        item_pack(
            uuid!("34a78027-ac6e-4bc6-856e-4b8cee5859be"),
            uuid!("4d790010-1a79-4bd0-a79b-d52cac068a3a"),
        ),
        // APEX PACK
        todo(uuid!("80a9babf-3088-4ce9-a986-804f6ce9660c")),
        // APEX POINTS
        todo(uuid!("3b2c8ed8-df9a-4659-aeda-786e06cc7dd9")),
        // LOYALTY PACK (ME3)
        todo(uuid!("47088308-e623-494e-a436-cccfd7f4150f")),
        // LOYALTY PACK (DA:I)
        todo(uuid!("523226d2-8a17-4081-9c22-71c890d1b4ab")),
        // BONUS REWARD PACK
        todo(uuid!("ab939baf-3cc0-46a8-8983-5c8e92754a25")),
        // PRE-ORDER BOOSTER PACK
        todo(uuid!("aa7b57df-d0a7-4275-8623-38575565fe15")),
        // ANDROMEDA INITIATIVE PACK
        todo(uuid!("9dba3f79-7c9f-4526-96f0-7eaec177eccf")),
        // SUPER DELUXE EDITION PACK - 1/20
        todo(uuid!("51e008c4-018c-477e-b99a-e8b44a86483b")),
        // SUPER DELUXE EDITION PACK - 2/20
        todo(uuid!("80304bc9-e704-4b5d-9193-e35f8de7b871")),
        // SUPER DELUXE EDITION PACK - 3/20
        todo(uuid!("efcc43cf-5877-4ef4-a52b-c35a88a154d2")),
        // SUPER DELUXE EDITION PACK - 4/20
        todo(uuid!("3ff3ff1b-d2f1-4912-9612-9c50cf7138e2")),
        // SUPER DELUXE EDITION PACK - 5/20
        todo(uuid!("22a72362-620c-4c86-bf83-83848336a6fb")),
        // SUPER DELUXE EDITION PACK - 6/20
        todo(uuid!("66e5a516-443c-4062-953c-d34ffec0e4c5")),
        // SUPER DELUXE EDITION PACK - 7/20
        todo(uuid!("06a249fd-324d-4a9e-9f46-7cb7e620652d")),
        // SUPER DELUXE EDITION PACK - 8/20
        todo(uuid!("384e4424-0421-4793-b713-13d68616505e")),
        // SUPER DELUXE EDITION PACK - 9/20
        todo(uuid!("e78760b4-2c64-45be-9906-e3183c64a424")),
        // SUPER DELUXE EDITION PACK - 10/20
        todo(uuid!("5baa0a3d-86e3-45cc-8ab1-d26591c46a3c")),
        // SUPER DELUXE EDITION PACK - 11/20
        todo(uuid!("03d7ec5a-d729-4fb3-91d2-2db11f8dfa40")),
        // SUPER DELUXE EDITION PACK - 12/20
        todo(uuid!("bed2b13e-1cca-4981-b81f-985c051565a4")),
        // SUPER DELUXE EDITION PACK - 13/20
        todo(uuid!("d21b1767-cb37-4bfa-ad30-12a9d2240775")),
        // SUPER DELUXE EDITION PACK - 14/20
        todo(uuid!("cbe39480-8473-4aa4-8a06-ce1524a5af2e")),
        // SUPER DELUXE EDITION PACK - 15/20
        todo(uuid!("317d54fd-0596-44ea-84ee-30b5fec1ab1d")),
        // SUPER DELUXE EDITION PACK - 16/20
        todo(uuid!("db74221c-1e7e-41af-9a20-cb8176d5d00b")),
        // SUPER DELUXE EDITION PACK - 17/20
        todo(uuid!("c1a96446-ae8e-47f5-8770-caeb69f862bd")),
        // SUPER DELUXE EDITION PACK - 18/20
        todo(uuid!("774be722-7814-4c72-9d6f-08e5bf98aa47")),
        // SUPER DELUXE EDITION PACK - 19/20
        todo(uuid!("b0fce148-f9d8-4098-b767-0e3e523f6e0d")),
        // SUPER DELUXE EDITION PACK - 20/20
        todo(uuid!("23f98283-f960-46d6-85f9-4bf85d60e2cd")),
        // APEX REINFORCEMENT PACK
        todo(uuid!("c4b1ebe3-e0b0-42fb-a51c-c6c2d688ac71")),
        // APEX COMMENDATION PACK
        todo(uuid!("203ce2dc-962f-44c8-a513-76ee2286d0b7")),
        // APEX CHALLENGE PACK
        todo(uuid!("17f90be7-8d74-4593-a85f-0b4cdb9f57ba")),
        // LOGITECH WEAPON PACK
        todo(uuid!("7f2a365a-9f08-412f-8490-ce55fd34aad6")),
        // BONUS BOOSTER PACK
        todo(uuid!("33cb8ec3-efce-4744-a858-db5e60e11424")),
        // SUPPORT PACK
        todo(uuid!("fcc1fbf1-fa53-445b-b2e9-561702795627")),
        // TOTINO'S BOOSTER PACK
        todo(uuid!("d8b62c9a-31f2-4e7e-82fe-43b9e72cbc7f")),
        // APEX HQ PACK
        todo(uuid!("8a072bab-e849-475d-b552-e18704b150c4")),
        // ADVANCED COMMUNITY PACK
        todo(uuid!("6fcbb0d5-b4ed-406d-8056-029ce7a91fd0")),
        // STARTER PACK
        todo(uuid!("cba5b757-cf67-40e1-a500-66dad3840088")),
        // TUTORIAL PACK
        todo(uuid!("37101bb8-e5c0-44d7-bcd9-bf49ceecc1de")),
        // DELUXE EDITION PACK
        todo(uuid!("cc15e17f-1b06-4413-9c6c-544d01b50f2a")),
        // NAMEPLATE: APEX MASTERY - BRONZE
        item_pack(
            uuid!("208aa537-19d0-4bea-9ac9-f11713cd85e8"),
            uuid!("dd241aa0-26ba-4165-8332-69ba6259a8d3"),
        ),
        // NAMEPLATE: APEX MASTERY - SILVER
        item_pack(
            uuid!("c9334ea7-9249-46a7-93af-b0622af5370e"),
            uuid!("ec666f35-cc51-4569-87ca-3c17ff25efe4"),
        ),
        // NAMEPLATE: APEX MASTERY - GOLD
        item_pack(
            uuid!("7ad4c7ea-2b31-412a-b688-c2d56619dcc3"),
            uuid!("dec5e82a-0151-4802-b9eb-064e1849cba1"),
        ),
        // NAMEPLATE: ASSAULT RIFLE MASTERY- BRONZE
        item_pack(
            uuid!("0b7386e1-3e9b-415e-b246-45d3674367f4"),
            uuid!("bcec3018-405b-4c52-86b5-d4aedacccbd7"),
        ),
        // NAMEPLATE: ASSAULT RIFLE MASTERY- SILVER
        item_pack(
            uuid!("0d31bf4b-3ab2-4d09-8028-335bb2f28ad8"),
            uuid!("fdd1d812-64e1-40e9-ad89-3b7f90641fab"),
        ),
        // NAMEPLATE: ASSAULT RIFLE MASTERY- GOLD
        item_pack(
            uuid!("19a680d4-5149-420a-aebe-03b9beb1ab83"),
            uuid!("1fa00e66-177d-4afb-831c-ca90fcf09e91"),
        ),
        // NAMEPLATE: COMBAT MASTERY - BRONZE
        item_pack(
            uuid!("d7e1823e-aa41-47fe-9602-13b6f31153f6"),
            uuid!("34a56ba9-1e06-4b27-8fb5-ca8122c6ac72"),
        ),
        // NAMEPLATE: COMBAT MASTERY - SILVER
        item_pack(
            uuid!("5d3d4ce8-9cf0-4ff6-9860-9e8554c10577"),
            uuid!("429c1c96-1aa6-4b9a-a109-754d4f1ce3ab"),
        ),
        // NAMEPLATE: COMBAT MASTERY - GOLD
        item_pack(
            uuid!("c537155c-efbd-49c2-a15c-2fcd088dfeb2"),
            uuid!("f958a50a-f9d4-477c-b071-d278fe6fa581"),
        ),
        // NAMEPLATE: KETT MASTERY- BRONZE
        item_pack(
            uuid!("f8a12dd0-dd4d-4151-91dc-7e019005a22c"),
            uuid!("26a31baf-8fef-4e8f-b704-29e9f335df0e"),
        ),
        // NAMEPLATE: KETT MASTERY- SILVER
        item_pack(
            uuid!("e1c4ff7d-63e5-4e82-ae89-a078b954edce"),
            uuid!("1d832caf-8ed5-4329-b33d-06d0ad9463f4"),
        ),
        // NAMEPLATE: KETT MASTERY- GOLD
        item_pack(
            uuid!("65e537a8-0a56-4ded-8d48-41e68d9d82cb"),
            uuid!("4d9c88f4-22d6-4096-8d5a-3e6629adf34f"),
        ),
        // NAMEPLATE: MAP MASTERY - BRONZE
        item_pack(
            uuid!("3dbc20f9-4258-44c8-aace-f89444f48346"),
            uuid!("59cbef6f-323b-47c2-93e1-a41bdef50d14"),
        ),
        // NAMEPLATE: MAP MASTERY - SILVER
        item_pack(
            uuid!("6d05ac99-3e2e-4f48-9b84-04c8d9be8420"),
            uuid!("8a3fbe71-eced-4d03-8cdc-f8ba3888b53c"),
        ),
        // NAMEPLATE: MAP MASTERY - GOLD
        item_pack(
            uuid!("ba606bb6-08b0-4002-b45e-ab0d07c4126d"),
            uuid!("129c6111-fdb8-4907-a820-8f9665de6d80"),
        ),
        // NAMEPLATE: OUTLAW MASTERY - BRONZE
        item_pack(
            uuid!("ce59f903-f3a1-4ec3-90a3-1e82c5f47b85"),
            uuid!("c2dd50c5-d650-4a75-bd49-f476a4e9d18e"),
        ),
        // NAMEPLATE: OUTLAW MASTERY - SILVER
        item_pack(
            uuid!("2d9e2f93-2c72-491e-bdb9-46f20d0d9339"),
            uuid!("713b03ba-cead-4cd7-8239-0ce38dbc32fb"),
        ),
        // NAMEPLATE: OUTLAW MASTERY - GOLD
        item_pack(
            uuid!("daf74c9a-8c2b-4de4-931f-dce265a88c1c"),
            uuid!("9223bffe-ce83-48bf-8eb5-ed9e7345bdaa"),
        ),
        // NAMEPLATE: APEX RATING - BRONZE
        item_pack(
            uuid!("5c7b9f32-4fef-430c-a72d-0e7409b84adc"),
            uuid!("80c863cc-d53f-4335-92bd-71d6cec3b08b"),
        ),
        // NAMEPLATE: APEX RATING - SILVER
        item_pack(
            uuid!("ad9c5a2f-63b0-4638-935c-1733f083de38"),
            uuid!("227809cc-1fdd-433a-83ea-0662778e36dd"),
        ),
        // NAMEPLATE: APEX RATING - GOLD
        item_pack(
            uuid!("74f437e4-fd7d-4f6a-a441-66e6c64bb3c5"),
            uuid!("07a2c3ed-269a-46a4-ab81-5aaa3ff586d8"),
        ),
        // NAMEPLATE: PISTOL MASTERY - BRONZE
        item_pack(
            uuid!("414b173e-2dcf-4587-8cdd-43c5bc872c5c"),
            uuid!("5fda99e2-93aa-4e62-a198-c1a4381d9b97"),
        ),
        // NAMEPLATE: PISTOL MASTERY - SILVER
        item_pack(
            uuid!("be469a8c-71d0-47f2-a13f-80c94beec052"),
            uuid!("23511ee2-1a01-4d4d-94ef-618a3c199b2b"),
        ),
        // NAMEPLATE: PISTOL MASTERY - GOLD
        item_pack(
            uuid!("73564b68-8e80-48b1-881c-2e2085787509"),
            uuid!("3164389f-46aa-4f10-b5cb-4c5839a00f57"),
        ),
        // NAMEPLATE: REMNANT MASTERY - BRONZE
        item_pack(
            uuid!("a6248be2-1647-4e9b-9e1e-b8b69ecf809d"),
            uuid!("561289b5-9efa-4d6f-acf4-ce8c2ff26792"),
        ),
        // NAMEPLATE: REMNANT MASTERY - SILVER
        item_pack(
            uuid!("123b3fa1-565e-456f-b08d-aa131b0c5cf1"),
            uuid!("4006a2e7-c0b5-4d02-b542-1c14ea05e9a4"),
        ),
        // NAMEPLATE: REMNANT MASTERY - GOLD
        item_pack(
            uuid!("206115c9-c953-4ce2-aab0-6804660f6cc1"),
            uuid!("9f571cb9-3846-41a0-a0c9-abc7dfac2772"),
        ),
        // NAMEPLATE: SHOTGUN MASTERY - BRONZE
        item_pack(
            uuid!("aa7b4129-1e67-421a-a3e9-27813bd1105a"),
            uuid!("771029a8-e7ed-46a5-af30-e87ee73350f1"),
        ),
        // NAMEPLATE: SHOTGUN MASTERY - SILVER
        item_pack(
            uuid!("88a7e312-1591-4ac5-bdd8-6be1a6f02c9f"),
            uuid!("bed37817-170d-4144-9434-3ccd58c7ec8f"),
        ),
        // NAMEPLATE: SHOTGUN MASTERY - GOLD
        item_pack(
            uuid!("fa6aab20-ae9a-4778-829b-978f075de939"),
            uuid!("4fa9a564-dfbd-4c28-8ba5-6e9e3e48d950"),
        ),
        // NAMEPLATE: SNIPER RIFLE MASTERY - BRONZE
        item_pack(
            uuid!("66e865bb-b694-4f2a-86e3-caf58442780d"),
            uuid!("2e0c84a8-0495-469e-a059-b71759cadf0a"),
        ),
        // NAMEPLATE: SNIPER RIFLE MASTERY - SILVER
        item_pack(
            uuid!("254dad07-4f5b-4ce0-9d78-6be17855f082"),
            uuid!("9945b0d6-2515-4329-a718-cfe1fb26b2d0"),
        ),
        // NAMEPLATE: SNIPER RIFLE MASTERY - GOLD
        item_pack(
            uuid!("d9e0d08d-5ffc-4e33-9509-40776591eb68"),
            uuid!("6282e95d-5b15-482d-96bc-060e34126177"),
        ),
        // NAMEPLATE: TECH MASTERY - BRONZE
        item_pack(
            uuid!("6d830d65-13de-4c70-8fb9-d076c569b4f0"),
            uuid!("153c87ec-0b2f-4cc1-9a84-4ad646d1418f"),
        ),
        // NAMEPLATE: TECH MASTERY - SILVER
        item_pack(
            uuid!("8fd74763-e397-45ab-a27a-ac8f08e062e1"),
            uuid!("beefc0ed-d91c-463e-bc2c-ade1c9927ab5"),
        ),
        // NAMEPLATE: TECH MASTERY - GOLD
        item_pack(
            uuid!("737be245-d4ae-410b-9bf8-3db805eb79b7"),
            uuid!("6dbd41ae-c394-4502-984b-228075eada9f"),
        ),
        // NAMEPLATE: BIOTIC MASTERY - BRONZE
        item_pack(
            uuid!("6b1179d1-0a7b-496c-83e2-f66de8b57736"),
            uuid!("70f12a9a-a979-4d62-bda1-5f161e8f133a"),
        ),
        // NAMEPLATE: BIOTIC MASTERY - SILVER
        item_pack(
            uuid!("e9d39579-0f21-4d35-952f-cd418b6c4b57"),
            uuid!("9288bbdb-c045-439c-8771-651b83c294cc"),
        ),
        // NAMEPLATE: BIOTIC MASTERY - GOLD
        item_pack(
            uuid!("8b9263f0-a660-48b3-8a83-f11cfb4da11b"),
            uuid!("c072a185-7173-4a4b-87ce-c76e2ac9cead"),
        ),
        // AESTHETIC
        todo(uuid!("53a5fc5e-3ba9-476f-a537-555bac6014f3")),
        todo(uuid!("8425ccb0-37f4-4d5e-915c-0806602f2593")),
        todo(uuid!("361895d8-49b0-4d0c-b359-60e7c343f194")),
        todo(uuid!("1e6627c8-f8ee-4c70-86b2-0c2dd4c65ff4")),
        todo(uuid!("c869e5a6-cb6c-4580-a162-d5ac3f72b737")),
        todo(uuid!("6e67e5e2-89c7-44cc-89fb-432e8e99734a")),
        todo(uuid!("55d1d22f-0ee7-41bf-939a-0aa372bb2e72")),
        todo(uuid!("e3f10da1-312a-4ba4-ad33-0c503e6c2a8f")),
        todo(uuid!("c9d603e7-9e20-4d72-a672-81c1a188a320")),
        // DELUXE EDITION PACK #2
        todo(uuid!("e57690fe-4b17-4b11-b1de-a1fd4b0b4a55")),
        // EA ACCESS PACK
        todo(uuid!("77459eda-2eab-4aae-b8f0-d26964f269eb")),
        // TECH TEST SIGN-UP - BRONZE
        todo(uuid!("e28207db-3b14-4ba7-9dc6-d0826d76b78d")),
        // ORIGIN ACCESS PACK
        todo(uuid!("7c4118cd-53fa-4c15-951c-6c250549db1d")),
        // SUPPORT PACK
        todo(uuid!("0d9a69e0-cad5-4242-8052-9f0c2ded0236")),
        // APEX ELITE PACK
        todo(uuid!("5e7cf499-4f72-47d8-b87b-04162ef4e406")),
        // MEA DEVELOPER - GOLD
        todo(uuid!("0b2986da-3d0d-45fd-b0b7-2adfca9d2994")),
        // CELEBRATORY PACK
        todo(uuid!("a883a017-1b11-41ea-b98a-127b25dd3032")),
        todo(uuid!("5aebef08-b14c-40df-95fe-59fc78274ad5")),
        // MP DLC PACK - COLLECTION ITEMS
        todo(uuid!("eed5b4df-736d-4b4c-b683-96c19dc5088d")),
        todo(uuid!("eb4fe1a6-c942-43f9-91f5-7b981ccbbb55")),
        todo(uuid!("ccb3f225-e808-4057-99b8-48a33c966be1")),
        todo(uuid!("ef8d85dc-74c5-4554-86c2-4e2f5c7e0fb8")),
        todo(uuid!("f1473ab2-55c1-4b22-a8d2-344dba5b4e09")),
        todo(uuid!("43eed42a-643a-4ddc-b0b7-51e6ed5ccbf8")),
        todo(uuid!("67416130-bd36-4cf4-94df-e276f7642472")),
        todo(uuid!("a1e73511-3672-40b0-9a9f-8c24faa8b831")),
        todo(uuid!("23b6647a-0b54-43a8-85fb-0a382522bf97")),
        todo(uuid!("609be685-d3c3-43a6-b0a1-484701c19172")),
        todo(uuid!("e4e12a1d-6f0a-4191-a740-26e715e42abe")),
        todo(uuid!("f8aecee2-3add-4b73-a520-961ef9932ea2")),
        // [BUG] I am a banner!
        todo(uuid!("694577c3-0d92-4e85-ad41-de54a4c91154")),
    ]
    .into_iter()
    .map(|pack| (pack.name, pack))
    .collect()
}
