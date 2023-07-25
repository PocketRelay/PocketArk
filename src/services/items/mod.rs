//! Service in charge of deailing with items opening packs

use crate::{database::entity::InventoryItem, utils::models::LocaleNameWithDesc};
use log::{debug, error};
use rand::{distributions::WeightedError, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, hash::Hash, process::exit, str::FromStr};
use thiserror::Error;
use uuid::{uuid, Uuid};

pub const INVENTORY_DEFINITIONS: &str =
    include_str!("../../resources/data/inventoryDefinitions.json");

pub struct ItemsService {
    /// Item definitions
    defs: Vec<ItemDefinition>,
    /// Lookup table for item definitions based on their name
    defs_by_name: HashMap<Uuid, usize>,
    /// Available unlock packs
    pub packs: HashMap<Uuid, Pack>,
}

impl ItemsService {
    pub fn new() -> Self {
        let defs: Vec<ItemDefinition> = match serde_json::from_str(INVENTORY_DEFINITIONS) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to load inventory definitions: {}", err);
                exit(1);
            }
        };

        let defs_by_name: HashMap<Uuid, usize> = defs
            .iter()
            .enumerate()
            .map(|(index, definition)| (definition.name, index))
            .collect();

        debug!("Loaded {} inventory item definition(s)", defs.len());

        let packs: HashMap<Uuid, Pack> = [
            // Packs
            Self::supply_pack(),
            Self::basic_pack(),
            Self::jumbo_supply_pack(),
            Self::ammo_priming_pack(),
            Self::technical_mods_pack(),
            Self::advanced_pack(),
            Self::expert_pack(),
            Self::reserves_pack(),
            Self::arsenal_pack(),
            Self::premium_pack(),
            Self::jumbo_premium_pack(),
            // Item store
            Self::bonus_reward_pack(),
            Self::random_common_mod_pack(),
            Self::random_uncommon_mod_pack(),
        ]
        .into_iter()
        .map(|pack| (pack.name, pack))
        .collect();

        Self {
            defs,
            defs_by_name,
            packs,
        }
    }

    pub fn defs(&self) -> &[ItemDefinition] {
        &self.defs
    }

    pub fn by_name(&self, name: &Uuid) -> Option<&ItemDefinition> {
        let index = self.defs_by_name.get(name).copied()?;
        let def = &self.defs[index];
        Some(def)
    }

    fn supply_pack() -> Pack {
        Pack::new("c5b3d9e6-7932-4579-ba8a-fd469ed43fda")
            // COBRA RPG
            .add_item(ItemChance::named(uuid!(
                "eaefec2a-d892-498b-a175-e5d2048ae39a"
            )))
            // REVIVE PACK
            .add_item(ItemChance::named(uuid!(
                "af39be6b-0542-4997-b524-227aa41ae2eb"
            )))
            // AMMO PACK
            .add_item(ItemChance::named(uuid!(
                "2cc0d932-8e9d-48a6-a6e8-a5665b77e835"
            )))
            // FIRST AID PACK
            .add_item(ItemChance::named(uuid!(
                "4d790010-1a79-4bd0-a79b-d52cac068a3a"
            )))
            // Random Boosters
            .add_item(ItemChance::new(ItemFilter::category(Category::BOOSTERS)))
    }

    fn basic_pack() -> Pack {
        Pack::new("c6d431eb-325f-4765-ab8f-e48d7b58aa36")
            // 4 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Common items
                    ItemFilter::rarity(Rarity::COMMON),
                    // Items or characters
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(4),
            )
            // 1 item/character that is uncommon or common
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Common with low chance of uncommon
                    ItemFilter::rarity(Rarity::COMMON).weight(8)
                        | ItemFilter::rarity(Rarity::UNCOMMON).weight(1),
                    // Items or characters
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(1),
            )
    }

    fn jumbo_supply_pack() -> Pack {
        Pack::new("e4f4d32a-90c3-4f5c-9362-3bb5933706c7")
            // 5x COBRA RPG
            .add_item(
                ItemChance::named(uuid!("eaefec2a-d892-498b-a175-e5d2048ae39a")).stack_size(5),
            )
            // 5x REVIVE PACK
            .add_item(
                ItemChance::named(uuid!("af39be6b-0542-4997-b524-227aa41ae2eb")).stack_size(5),
            )
            // 5x AMMO PACK
            .add_item(
                ItemChance::named(uuid!("2cc0d932-8e9d-48a6-a6e8-a5665b77e835")).stack_size(5),
            )
            // 5x FIRST AID PACK
            .add_item(
                ItemChance::named(uuid!("4d790010-1a79-4bd0-a79b-d52cac068a3a")).stack_size(5),
            )
            // 5 Random Boosters
            .add_item(ItemChance::new(ItemFilter::category(Category::BOOSTERS)).amount(5))
    }

    // "Contains 2 of each Uncommon ammo booster, plus 2 additional boosters, at least 1 of which is Rare or better."
    fn ammo_priming_pack() -> Pack {
        Pack::new("eddfd7b7-3476-4ad7-9302-5cfe77ee4ea6")
            // 4 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Common items
                    ItemFilter::rarity(Rarity::UNCOMMON),
                    // Items or characters (weighted for weapons)
                    ItemFilter::and(
                        ItemFilter::category(Category::BOOSTERS),
                        ItemFilter::attributes([("consumableType", "Ammo")]),
                    ),
                ))
                // TODO: No way of specifiying one of EACH so all items not just an amount
                .amount(4),
            )
            .add_item(ItemChance::new(ItemFilter::category(Category::BOOSTERS)))
            .add_item(ItemChance::new(ItemFilter::and(
                // Common with low chance of uncommon
                ItemFilter::rarity(Rarity::RARE).weight(8)
                    | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                // Items or characters (weighted for weapons)
                ItemFilter::category(Category::BOOSTERS),
            )))
    }

    fn technical_mods_pack() -> Pack {
        Pack::new("975f87f5-0242-4c73-9e0f-6e4033b22ee9")
            // 4 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Exclude ultra rare and rare items from first selection
                    ItemFilter::rarity(Rarity::COMMON),
                    // Items or characters (weighted for characters)
                    ItemFilter::category(Category::CONSUMABLE)
                        | ItemFilter::category(Category::WEAPON_MODS)
                        | ItemFilter::category(Category::WEAPON_MODS_ENHANCED),
                ))
                .amount(4),
            )
            // 1 item/character that are rare or greater
            .add_item(ItemChance::new(ItemFilter::and(
                // Uncommon wiht a chance for rare
                ItemFilter::rarity(Rarity::UNCOMMON).weight(8)
                    | ItemFilter::rarity(Rarity::RARE).weight(1),
                // Items or characters (weighted for characters)
                ItemFilter::category(Category::CONSUMABLE)
                    | ItemFilter::category(Category::WEAPON_MODS)
                    | ItemFilter::category(Category::WEAPON_MODS_ENHANCED),
            )))
    }

    fn advanced_pack() -> Pack {
        Pack::new("974a8c8e-08bc-4fdb-bede-43337c255df8")
            // 4 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(4),
            )
            // 1 item/character that are rare or greater
            .add_item(ItemChance::new(ItemFilter::and(
                ItemFilter::rarity(Rarity::UNCOMMON).weight(8)
                    | ItemFilter::rarity(Rarity::RARE).weight(1),
                ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
            )))
    }

    fn expert_pack() -> Pack {
        Pack::new("b6fe6a9f-de70-463a-bcc5-a1b146067470")
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON) | ItemFilter::rarity(Rarity::UNCOMMON),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(4),
            )
            .add_item(ItemChance::new(ItemFilter::and(
                ItemFilter::rarity(Rarity::RARE).weight(8)
                    | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
            )))
    }

    fn reserves_pack() -> Pack {
        Pack::new("731b16c9-3a97-4166-a2f7-e79c8b45128a")
            // 3 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Exclude ultra rare and rare items from first selection
                    !(ItemFilter::rarity(Rarity::RARE) | ItemFilter::rarity(Rarity::ULTRA_RARE)),
                    // Items or characters (weighted for characters)
                    ItemFilter::categories(Category::ITEMS)
                        | ItemFilter::category(Category::CHARACTERS).weight(2),
                ))
                .amount(3),
            )
            // 2 item/character that are rare or greater
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Rare or greater
                    ItemFilter::rarity(Rarity::RARE).weight(2)
                        | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                    // Items or characters (weighted for characters)
                    ItemFilter::categories(Category::ITEMS)
                        | ItemFilter::category(Category::CHARACTERS).weight(2),
                ))
                .amount(2),
            )
    }

    fn arsenal_pack() -> Pack {
        Pack::new("29c47d42-5830-435b-943f-bf6cf04145e1")
            // 3 common items/weapons
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Exclude ultra rare and rare items from first selection
                    !(ItemFilter::rarity(Rarity::RARE) | ItemFilter::rarity(Rarity::ULTRA_RARE)),
                    // Items or characters (weighted for weapons)
                    ItemFilter::categories(Category::ITEMS_NO_WEAPONS)
                        | ItemFilter::category(Category::WEAPONS).weight(2),
                ))
                .amount(3),
            )
            // 2 item/weapons that are rare or greater
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Rare or greater
                    ItemFilter::rarity(Rarity::RARE).weight(2)
                        | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                    // Items or characters (weighted for weapons)
                    ItemFilter::categories(Category::ITEMS_NO_WEAPONS)
                        | ItemFilter::category(Category::WEAPONS).weight(2),
                ))
                .amount(2),
            )
    }

    fn premium_pack() -> Pack {
        Pack::new("8344cd62-2aed-468d-b155-6ae01f1f2405")
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON) | ItemFilter::rarity(Rarity::UNCOMMON),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(3),
            )
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::RARE).weight(4)
                        | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(2),
            )
    }
    fn jumbo_premium_pack() -> Pack {
        Pack::new("e3e56e89-b995-475f-8e75-84bf27dc8297")
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON) | ItemFilter::rarity(Rarity::UNCOMMON),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(10),
            )
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::RARE).weight(8)
                        | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(10),
            )
            .add_item(
                ItemChance::new(ItemFilter::and(
                    ItemFilter::rarity(Rarity::COMMON).weight(4)
                        | ItemFilter::rarity(Rarity::UNCOMMON).weight(4)
                        | ItemFilter::rarity(Rarity::RARE).weight(2)
                        | ItemFilter::rarity(Rarity::ULTRA_RARE).weight(1),
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(5),
            )
    }

    fn bonus_reward_pack() -> Pack {
        Pack::new("cf9cd252-e1f2-4574-973d-d66cd81558d3")
            // 3 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Exclude ultra rare and rare items from first selection
                    ItemFilter::rarity(Rarity::COMMON),
                    // Items or characters (weighted for weapons)
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(4),
            )
            // 1 maybe uncommon item/character
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Uncommon but with a chance for rare
                    ItemFilter::rarity(Rarity::UNCOMMON).weight(6)
                        | ItemFilter::rarity(Rarity::RARE).weight(1),
                    // Items or characters (weighted for weapons)
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(1),
            )
            // 1 maybe rare item/character
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Uncommon but with a chance for rare
                    ItemFilter::rarity(Rarity::COMMON).weight(6)
                        | ItemFilter::rarity(Rarity::RARE).weight(1),
                    // Items or characters (weighted for weapons)
                    ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
                ))
                .amount(1),
            )
    }

    fn random_uncommon_mod_pack() -> Pack {
        Pack::new("44da78e5-8ceb-4684-983e-794329d4a631")
            // 3 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Uncommon items
                    ItemFilter::rarity(Rarity::UNCOMMON),
                    // Weapon mods
                    ItemFilter::category(Category::WEAPON_MODS)
                        | ItemFilter::category(Category::WEAPON_MODS_ENHANCED),
                ))
                .amount(1),
            )
    }

    fn random_common_mod_pack() -> Pack {
        Pack::new("890b2aa6-191f-4162-ae79-a78d23e3c505")
            // 3 common items/characters
            .add_item(
                ItemChance::new(ItemFilter::and(
                    // Uncommon items
                    ItemFilter::rarity(Rarity::COMMON),
                    // Weapon mods
                    ItemFilter::category(Category::WEAPON_MODS)
                        | ItemFilter::category(Category::WEAPON_MODS_ENHANCED),
                ))
                .amount(1),
            )
    }
}

pub struct Rarity {}

impl Rarity {
    pub const COMMON: &str = "0";
    pub const UNCOMMON: &str = "1";
    pub const RARE: &str = "2";
    pub const ULTRA_RARE: &str = "3";
}

pub struct Category;

#[allow(unused)]
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

    pub const ITEMS_NO_WEAPONS: &[&'static str] = &[
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
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
    name: Uuid,
    items: Vec<ItemChance>,
}

impl Pack {
    pub fn new(name: &str) -> Self {
        Self {
            items: Vec::new(),
            name: Uuid::from_str(name).expect("Invalid pack uuid"),
        }
    }

    fn add_item(mut self, chance: ItemChance) -> Self {
        self.items.push(chance);
        self
    }

    pub fn grant_items(
        &self,
        rng: &mut StdRng,
        items: &'static [ItemDefinition],
        owned_items: &[(InventoryItem, &'static ItemDefinition)],
        out: &mut Vec<GrantedItem>,
    ) -> Result<(), RandomError> {
        for chance in &self.items {
            let mut has_weights = false;

            // List of all items that can be dropped and match the chance filter
            let values: Vec<(&'static ItemDefinition, u32)> = items
                .iter()
                .filter(|value| value.droppable.unwrap_or_default())
                .filter(|value| {
                    let unlock_definition = match value.unlock_definition.as_ref() {
                        Some(value) => value,
                        // Item doesn't have an unlock definition
                        None => return true,
                    };

                    let (item, definition) = match owned_items
                        .iter()
                        .find(|(_, definition)| definition.name.eq(unlock_definition))
                    {
                        Some(value) => value,
                        // Player didn't own the required item
                        None => return false,
                    };

                    if let Some(cap) = definition.cap {
                        if item.stack_size != cap {
                            return false;
                        }
                    }

                    true
                })
                .filter_map(|value| {
                    let (check, weight) = chance.filter.check(value);
                    if check {
                        if weight > 0 {
                            has_weights = true
                        }

                        Some((value, weight))
                    } else {
                        None
                    }
                })
                .collect();

            // Randomly select items
            let items = if has_weights {
                values.choose_multiple_weighted(rng, chance.amount, |(_, weight)| *weight)?
            } else {
                values.choose_multiple(rng, chance.amount)
            };

            for (defintion, _) in items {
                let existing = out.iter_mut().find(|value| value.defintion.eq(defintion));

                if let Some(existing) = existing {
                    existing.stack_size += chance.stack_size;
                } else {
                    out.push(GrantedItem {
                        defintion,
                        stack_size: chance.stack_size,
                    })
                }
            }
        }

        Ok(())
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

#[derive(Debug)]
pub struct ItemChanged {
    pub item_id: Uuid,
    pub prev_stack_size: u32,
    pub stack_size: u32,
}

#[derive(Debug, Error)]
pub enum RandomError {
    #[error(transparent)]
    Weight(#[from] WeightedError),
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

    pub fn named(name: Uuid) -> Self {
        Self::new(ItemFilter::named(name))
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
    Named(Uuid),
    /// Filter requiring a rarity
    Rarity(String),
    /// Filter requiring a category
    Category(String),
    // Filter on item attributes
    Attributes(HashMap<String, Value>),

    Weighted {
        filter: Box<ItemFilter>,
        weight: u32,
    },

    /// Filter allowing any of the provided filters passing
    Any(Vec<ItemFilter>),
    /// Filter by both filters
    And(Box<ItemFilter>, Box<ItemFilter>),
    /// Filter by one or the other filters
    Or(Box<ItemFilter>, Box<ItemFilter>),
    /// Filter items that are not of a filter
    Not(Box<ItemFilter>),
}

impl std::ops::BitOr<ItemFilter> for ItemFilter {
    type Output = ItemFilter;
    fn bitor(self, rhs: ItemFilter) -> Self::Output {
        ItemFilter::Or(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::BitAnd<ItemFilter> for ItemFilter {
    type Output = ItemFilter;
    fn bitand(self, rhs: ItemFilter) -> Self::Output {
        ItemFilter::And(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Not for ItemFilter {
    type Output = ItemFilter;
    fn not(self) -> Self::Output {
        ItemFilter::Not(Box::new(self))
    }
}

#[allow(unused)]
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

    pub fn attributes<I, K, V>(values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        Self::Attributes(
            values
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }

    pub fn named(value: Uuid) -> Self {
        ItemFilter::Named(value)
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
            ItemFilter::And(left, right) => {
                let (l, w1) = left.check(item);
                let (r, w2) = right.check(item);

                (l && r, w1 + w2)
            }
            ItemFilter::Or(left, right) => {
                let (l, w1) = left.check(item);
                let (r, w2) = right.check(item);
                (l || r, if l { w1 } else { w2 })
            }
            ItemFilter::Named(name) => (name.eq(&item.name), 0),
            ItemFilter::Weighted { filter, weight } => {
                let (c, w) = filter.check(item);

                (c, w + *weight)
            }
            ItemFilter::Not(filter) => {
                let (result, weight) = filter.check(item);

                (!result, weight)
            }
            ItemFilter::Attributes(map) => {
                for (key, value) in map {
                    if !item
                        .custom_attributes
                        .get(key)
                        .is_some_and(|attr| value.eq(attr))
                    {
                        return (false, 0);
                    }
                }
                (true, 0)
            }
        }
    }

    pub fn and(left: ItemFilter, right: ItemFilter) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }
    pub fn or(left: ItemFilter, right: ItemFilter) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }

    pub fn not(filter: ItemFilter) -> Self {
        Self::Not(Box::new(filter))
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

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefinition {
    pub name: Uuid,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,

    pub custom_attributes: HashMap<String, Value>,
    #[serialize_always]
    pub secret: Option<Value>,
    pub category: String,
    pub attachable_categories: Vec<String>,
    pub rarity: Option<String>,
    pub droppable: Option<bool>,
    pub consumable: Option<bool>,
    pub cap: Option<u32>,

    /// Name of definition that this item depends on
    /// (Requires the item to reach its capacity before it can be dropped)
    /// TODO: Handle this when doing store rewards
    pub unlock_definition: Option<Uuid>,

    #[serde(flatten)]
    pub events: ItemEvents,

    pub restrictions: Option<String>,
    pub default_namespace: String,
}

/// Activity events that should be created when
/// different things happen to the item
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEvents {
    pub on_consume: Option<Vec<Value>>,
    pub on_add: Option<Vec<Value>>,
    pub on_remove: Option<Vec<Value>>,
}

impl PartialEq for ItemDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for ItemDefinition {}

impl Hash for ItemDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}
