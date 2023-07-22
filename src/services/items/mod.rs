//! Service in charge of deailing with items opening packs

use std::collections::HashMap;

use rand::rngs::StdRng;
use sea_orm::DatabaseTransaction;

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
}

pub struct PackBuilder {}

struct GuarenteeItem {
    def: String,
    stack_size: u32,
}

impl GuarenteeItem {
    async fn grant_item(
        &self,
        rng: &mut StdRng,
        user: &User,
        tx: &DatabaseTransaction,
    ) -> DbResult<InventoryItem> {
        let mut item =
            InventoryItem::create_or_append(tx, user, self.def.to_string(), self.stack_size)
                .await?;
        item.stack_size = self.stack_size;

        Ok(item)
    }
}

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
    pub const ASSAULT_RIFLE: &str = "1:AssaultRifle";
    pub const PISTOL: &str = "1:Pistol";
    pub const SHOTGUN: &str = "1:Shotgun";
    pub const SNIPER_RIFLE: &str = "1:SniperRifle";

    pub const WEAPONS: &[&'static str] = &[
        Self::ASSAULT_RIFLE,
        Self::PISTOL,
        Self::SHOTGUN,
        Self::SNIPER_RIFLE,
    ];

    // Weapon mods
    pub const ASSAULT_RIFLE_MODS: &str = "2:AssaultRifle";
    pub const PISTOL_MODS: &str = "2:Pistol";
    pub const SHOTGUN_MODS: &str = "2:Shotgun";
    pub const SNIPER_RIFLE_MODS: &str = "2:SniperRifle";

    pub const WEAPON_MODS: &[&'static str] = &[
        Self::ASSAULT_RIFLE_MODS,
        Self::PISTOL_MODS,
        Self::SHOTGUN_MODS,
        Self::SNIPER_RIFLE_MODS,
    ];

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
    pub const ASSAULT_RIFLE_SPECIALIZED: &str = "13:AssaultRifle";
    pub const PISTOL_SPECIALIZED: &str = "13:Pistol";
    pub const SHOTGUN_SPECIALIZED: &str = "13:Shotgun";
    pub const SNIPER_RIFLE_SPECIALIZED: &str = "13:SniperRifle";
    pub const WEAPONS_SPECIALIZED: &[&'static str] = &[
        Self::ASSAULT_RIFLE_SPECIALIZED,
        Self::PISTOL_SPECIALIZED,
        Self::SHOTGUN_SPECIALIZED,
        Self::SNIPER_RIFLE_SPECIALIZED,
    ];

    // Enhanced weapon mod variants
    pub const ASSAULT_RIFLE_MODS_ENHANCED: &str = "14:AssaultRifle";
    pub const PISTOL_MODS_ENHANCED: &str = "14:Pistol";
    pub const SHOTGUN_MODS_ENHANCED: &str = "14:Shotgun";
    pub const SNIPER_RIFLE_MODS_ENHANCED: &str = "14:SniperRifle";

    pub const WEAPON_MODS_ENHANCED: &[&'static str] = &[
        Self::ASSAULT_RIFLE_MODS_ENHANCED,
        Self::PISTOL_MODS_ENHANCED,
        Self::SHOTGUN_MODS_ENHANCED,
        Self::SNIPER_RIFLE_MODS_ENHANCED,
    ];

    pub const ITEMS: &[&'static str] = &[
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
        Self::ASSAULT_RIFLE,
        Self::PISTOL,
        Self::SHOTGUN,
        Self::SNIPER_RIFLE,
        Self::ASSAULT_RIFLE_MODS,
        Self::PISTOL_MODS,
        Self::SHOTGUN_MODS,
        Self::SNIPER_RIFLE_MODS,
        Self::ASSAULT_RIFLE_SPECIALIZED,
        Self::PISTOL_SPECIALIZED,
        Self::SHOTGUN_SPECIALIZED,
        Self::SNIPER_RIFLE_SPECIALIZED,
        Self::ASSAULT_RIFLE_MODS_ENHANCED,
        Self::PISTOL_MODS_ENHANCED,
        Self::SHOTGUN_MODS_ENHANCED,
        Self::SNIPER_RIFLE_MODS_ENHANCED,
    ];
}

pub struct ItemChance {
    filter: ItemFilter,
    amount: usize,
}

fn random_item_or_character() {
    // First 4 items must be common
    let first = ItemFilter::Rarity(Rarity::COMMON).and(
        // Can be items
        ItemFilter::categories(Category::ITEMS)
            // or characters
            .or(ItemFilter::Category(Category::CHARACTERS)),
    );

    let second = ItemFilter::Rarity(Rarity::UNCOMMON);

    let filter = ItemFilter::any([ItemFilter::Category("")]);
}

struct RandomItem {
    filter: ItemFilter,
    stack_size: u32,
}

impl RandomItem {
    async fn grant_item(
        &self,
        rng: &mut StdRng,
        user: &User,
        tx: &DatabaseTransaction,
        items: &[ItemDefinition],
    ) -> DbResult<InventoryItem> {
        let weights: HashMap<String, u32> = HashMap::new();
        let items: Vec<&'static ItemDefinition> = items
            .iter()
            .filter_map(|value| {
                let (result, weight) = self.filter.check(value);
                if result {
                    weights.insert(value.name.to_string(), weight);
                    Some(value)
                } else {
                    None
                }
            })
            .collect();

        Ok(item)
    }
}

impl PackBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

pub enum ItemFilter {
    None,

    /// Literal name of the item definition to use
    Named(&'static str),
    /// Filter requiring a rarity
    Rarity(&'static str),
    /// Filter requiring a category
    Category(&'static str),

    /// Weighted filtering
    Weighted(Box<ItemFilter>, u32),

    /// Filter allowing any of the provided filters passing
    Any(Vec<ItemFilter>),
    /// Filter by both filters
    And(Box<ItemFilter>, Box<ItemFilter>),
    /// Filter by one or the other filters
    Or(Box<ItemFilter>, Box<ItemFilter>),
}

impl ItemFilter {
    pub fn categories(values: &[&'static str]) -> Self {
        Self::Any(
            values
                .iter()
                .map(|value| ItemFilter::Category(*value))
                .collect(),
        )
    }

    pub fn rarities(values: &[&'static str]) -> Self {
        Self::Any(
            values
                .iter()
                .map(|value| ItemFilter::Rarity(*value))
                .collect(),
        )
    }

    pub fn check(&self, item: &ItemDefinition) -> (bool, u32) {
        let result = match self {
            ItemFilter::Rarity(rarity) => {
                item.rarity.as_ref().is_some_and(|value| value.eq(rarity))
            }
            ItemFilter::Category(category) => item.category.eq(category),
            ItemFilter::Any(values) => values.iter().any(|value| value.check(item)),
            ItemFilter::And(first, second) => first.check(item) && second.check(item),
            ItemFilter::Or(first, second) => first.check(item) || second.check(item),
            ItemFilter::None => true,
            ItemFilter::Named(name) => name.eq(&item.name),
            ItemFilter::Weighted(left, weight) => return (left.check(item), weight),
        };
        (result, 0)
    }

    pub fn and(self, other: ItemFilter) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }
    pub fn or(self, other: ItemFilter) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    pub fn any<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Any(iter.into_iter().collect())
    }
}
