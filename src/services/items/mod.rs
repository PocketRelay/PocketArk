//! Service in charge of deailing with items opening packs

use std::{collections::HashMap, fs::File, process::exit};

use log::{debug, error};
use rand::{distributions::WeightedError, rngs::StdRng, seq::SliceRandom, SeedableRng};
use sea_orm::DatabaseTransaction;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    database::{
        entity::{InventoryItem, User},
        DbResult,
    },
    http::models::inventory::ItemDefinition,
};

use super::defs::LookupList;

pub const INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

pub struct ItemsService {
    pub inventory: LookupList<String, ItemDefinition>,
    pub packs: HashMap<String, Pack>,
}

impl ItemsService {
    pub fn load() -> Self {
        let list: Vec<ItemDefinition> = match serde_json::from_str(INVENTORY_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load inventory definitions: {}", err);
                exit(1);
            }
        };

        debug!("Loaded {} inventory item definition(s)", list.len());

        let inventory = LookupList::create(list, |value| value.name.to_string());
        let mut packs = HashMap::new();

        Self { inventory, packs }
    }

    pub fn test(&'static self) {
        let pack = Pack::new()
            // 4 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(4),
            )
            // 1 item/character thats maybe uncommon
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::or(
                        ItemFilter::rarity(Rarity::COMMON).weight(5),
                        ItemFilter::rarity(Rarity::UNCOMMON).weight(1),
                    ),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(1),
            );

        let mut map = HashMap::new();
        map.insert(Uuid::new_v4(), pack);

        let mut out = File::create("packs.json").unwrap();
        let out = serde_json::to_writer(&mut out, &map).unwrap();

        // let mut rng = StdRng::from_entropy();

        // let mut out = Vec::new();
        // pack.grant_items(&mut rng, self.inventory.list(), &mut out);

        // debug!("Test Grant: {:#?}", out);
    }
}

pub struct PackBuilder {}

struct Rarity {}

impl Rarity {
    pub const COMMON: &str = "0";
    pub const UNCOMMON: &str = "1";
    pub const RARE: &str = "2";
    pub const ULTRA_RARE: &str = "3";
}

struct Category;

impl Category {
    /// Character items
    pub const CHARACTERS: &str = "0";

    // Weapons
    pub const WEAPONS: &str = "1:";
    pub const ASSAULT_RIFLE: &str = "1:AssaultRifle";
    pub const PISTOL: &str = "1:Pistol";
    pub const SHOTGUN: &str = "1:Shotgun";
    pub const SNIPER_RIFLE: &str = "1:SniperRifle";

    // Weapon mods
    pub const WEAPON_MODS: &str = "2:";
    pub const ASSAULT_RIFLE_MODS: &str = "2:AssaultRifle";
    pub const PISTOL_MODS: &str = "2:Pistol";
    pub const SHOTGUN_MODS: &str = "2:Shotgun";
    pub const SNIPER_RIFLE_MODS: &str = "2:SniperRifle";

    /// Boosters such as "AMMO CAPACITY MOD I", "ASSAULT RIFLE RAIL AMP", "CRYO AMMO"
    pub const BOOSTERS: &str = "3";

    // Consumable items such as "AMMO PACK", "COBTRA RPG", "REVIVE PACK"
    pub const CONSUMABLE: &str = "4";

    /// Equipment such as "ADAPTIVE WAR AMP", and "ASSAULT LOADOUT"
    pub const EQUIPMENT: &str = "5";

    /// Rewards from challenges
    pub const CHALLENGE_REWARD: &str = "7";

    /// Non droppable rewards for apex points
    pub const APEX_POINTS: &str = "8";

    /// Upgrades for capacity such as "AMMO PACK CAPACITY INCREASE" and
    /// "CHARACTER RESPEC" items
    pub const CAPACITY_UPGRADE: &str = "9";

    /// Rewards from strike team missions (Loot boxes)
    pub const STRIKE_TEAM_REWARD: &str = "11";

    /// Item loot box packs
    pub const ITEM_PACK: &str = "12";

    // Specialized gun variants
    pub const WEAPONS_SPECIALIZED: &str = "13:";
    pub const ASSAULT_RIFLE_SPECIALIZED: &str = "13:AssaultRifle";
    pub const PISTOL_SPECIALIZED: &str = "13:Pistol";
    pub const SHOTGUN_SPECIALIZED: &str = "13:Shotgun";
    pub const SNIPER_RIFLE_SPECIALIZED: &str = "13:SniperRifle";

    // Enhanced weapon mod variants
    pub const WEAPON_MODS_ENHANCED: &str = "14:";
    pub const ASSAULT_RIFLE_MODS_ENHANCED: &str = "14:AssaultRifle";
    pub const PISTOL_MODS_ENHANCED: &str = "14:Pistol";
    pub const SHOTGUN_MODS_ENHANCED: &str = "14:Shotgun";
    pub const SNIPER_RIFLE_MODS_ENHANCED: &str = "14:SniperRifle";

    pub const ITEMS: &[&'static str] = &[
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
        Self::WEAPONS,
        Self::WEAPON_MODS,
        Self::WEAPONS_SPECIALIZED,
        Self::WEAPON_MODS_ENHANCED,
    ];
    pub const ITEMS_WITH_CHARACTERS: &[&'static str] = &[
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
        Self::WEAPONS,
        Self::WEAPON_MODS,
        Self::WEAPONS_SPECIALIZED,
        Self::WEAPON_MODS_ENHANCED,
        Self::CHARACTERS,
    ];
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pack {
    items: Vec<ItemChance>,
}

impl Pack {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add_item(mut self, chance: ItemChance) -> Self {
        self.items.push(chance);
        self
    }

    fn grant_items(
        &self,
        rng: &mut StdRng,
        items: &'static [ItemDefinition],
        out: &mut Vec<GrantedItem>,
    ) -> Result<(), RandomError> {
        let values = self.items_by_filter(items, &self.items);

        for chance in &self.items {
            out.reserve_exact(chance.amount);

            let items =
                values.choose_multiple_weighted(rng, chance.amount, |(_, _, weight)| *weight)?;

            for (defintion, chance, _) in items {
                let item = GrantedItem {
                    defintion,
                    stack_size: chance.stack_size,
                };
                out.push(item)
            }
        }

        Ok(())
    }

    fn items_by_filter<'a>(
        &self,
        defs: &'static [ItemDefinition],
        items: &'a [ItemChance],
    ) -> Vec<(&'static ItemDefinition, &'a ItemChance, u32)> {
        // TODO: Provide list of user unlocks to check for X and S variants unlockDefinitions

        defs.iter()
            .filter(|value| value.droppable.unwrap_or_default())
            .filter_map(|value| {
                for item in items {
                    let (check, weight) = item.filter.check(value);
                    if check {
                        return Some((value, item, weight));
                    }
                }

                None
            })
            .collect()
    }
}

/// Represents an item thats been granted
#[derive(Debug)]
pub struct GrantedItem {
    /// The item definition
    pub defintion: &'static ItemDefinition,
    /// The total number of items to grant
    pub stack_size: u32,
}

#[derive(Debug, Error)]
pub enum RandomError {
    #[error(transparent)]
    Weight(#[from] WeightedError),
}

impl PackBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemChance {
    pub filter: ItemFilter,
    pub stack_size: u32,
    pub amount: usize,
}

impl ItemChance {
    pub fn new(filter: ItemFilter) -> Self {
        Self {
            filter,
            stack_size: 1,
            amount: 1,
        }
    }

    pub fn amount(mut self, amount: usize) -> Self {
        self.amount = amount;
        self
    }

    pub fn stack_size(mut self, stack_size: u32) -> Self {
        self.stack_size = stack_size;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemFilter {
    /// Literal name of the item definition to use
    Named(String),
    /// Filter requiring a rarity
    Rarity(String),
    /// Filter requiring a category
    Category(String),

    Weighted {
        filter: Box<ItemFilter>,
        weight: u32,
    },

    /// Filter allowing any of the provided filters passing
    Any(Vec<ItemFilter>),
    /// Filter by both filters
    And {
        left: Box<ItemFilter>,
        right: Box<ItemFilter>,
    },
    /// Filter by one or the other filters
    Or {
        left: Box<ItemFilter>,
        right: Box<ItemFilter>,
    },
}

impl ItemFilter {
    pub fn categories(values: &[&str]) -> Self {
        Self::Any(
            values
                .iter()
                .map(|value| ItemFilter::Category(value.to_string()))
                .collect(),
        )
    }

    pub fn rarities(values: &[&str]) -> Self {
        Self::Any(
            values
                .iter()
                .map(|value| ItemFilter::Rarity(value.to_string()))
                .collect(),
        )
    }

    pub fn named(value: &str) -> Self {
        ItemFilter::Named(value.to_string())
    }

    pub fn rarity(value: &str) -> Self {
        ItemFilter::Rarity(value.to_string())
    }

    pub fn category(value: &str) -> Self {
        ItemFilter::Category(value.to_string())
    }

    pub fn check(&self, item: &ItemDefinition) -> (bool, u32) {
        match self {
            ItemFilter::Rarity(rarity) => (
                item.rarity.as_ref().is_some_and(|value| value.eq(rarity)),
                0,
            ),
            ItemFilter::Category(category) => {
                let check = if category.ends_with(':') {
                    item.category.starts_with(category)
                } else {
                    item.category.eq(category)
                };

                (check, 0)
            }
            ItemFilter::Any(values) => {
                let mut total_weight = 0;
                let mut matches = false;

                for value in values {
                    let (result, weight) = value.check(item);
                    total_weight += weight;
                    if result {
                        matches = true;
                    }
                }

                (matches, total_weight)
            }
            ItemFilter::And { left, right } => {
                let (l, w1) = left.check(item);
                let (r, w2) = right.check(item);

                (l && r, w1 + w2)
            }
            ItemFilter::Or { left, right } => {
                let (l, w1) = left.check(item);
                let (r, w2) = right.check(item);
                (l || r, if l { w1 } else { w2 })
            }
            ItemFilter::Named(name) => (name.eq(&item.name), 0),
            ItemFilter::Weighted { filter, weight } => {
                let (c, w) = filter.check(item);

                (c, w + *weight)
            }
        }
    }

    pub fn and(left: ItemFilter, right: ItemFilter) -> Self {
        Self::And {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn or(left: ItemFilter, right: ItemFilter) -> Self {
        Self::Or {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn weight(self, weight: u32) -> Self {
        Self::Weighted {
            filter: Box::new(self),
            weight,
        }
    }

    pub fn any<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Any(iter.into_iter().collect())
    }
}
