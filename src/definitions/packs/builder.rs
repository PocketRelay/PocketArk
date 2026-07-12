//! # Pack Builder
//!
//! Pack builder using code to generate packs, used to generate the initial
//! packs.json that can be modified
#![allow(unused)]

use super::filter::Filter;
use crate::definitions::{
    items::{BaseCategory, Category, ItemName, ItemRarity},
    packs::{Pack, PackCollection},
};
use std::collections::HashMap;
use uuid::uuid;

/// Builder for creating [Pack]s
pub struct PackBuilder {
    /// The name of the pack item
    name: ItemName,

    /// Description of the pack / in game name
    description: String,

    /// Description of the pack contents (For internal reference)
    contents_description: String,

    /// The collection of item reward this pack provides
    collections: Vec<PackCollection>,
}

impl PackBuilder {
    /// Creates a new pack builder using the provided name
    pub fn new(name: ItemName, description: String, contents_description: String) -> Self {
        Self {
            name,
            collections: Vec::new(),
            description,
            contents_description,
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
            description: self.description,
            contents_description: self.contents_description,
            collections: self.collections.into_boxed_slice(),
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

    //
    let supply_pack = Pack::builder(uuid!("c5b3d9e6-7932-4579-ba8a-fd469ed43fda"), "Supply Pack", "Includes the Cobra RPG, First Aid Pack, Ammo Pack, and Revive Pack, as well as a Random Booster.")
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

    //
    let basic_pack = Pack::builder(
        uuid!("c6d431eb-325f-4765-ab8f-e48d7b58aa36"),
        "Basic Pack",
        "Contains 5 random items or characters, with a small chance that 1 will be Uncommon",
    )
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

    //
    let jumbo_supply_pack = Pack::builder(uuid!("e4f4d32a-90c3-4f5c-9362-3bb5933706c7"), "Jumbo Supply Pack", "Includes 5 each of the Cobra RPG, First Aid Pack, Ammo Pack, and Revive Pack, as well as 5 Random Boosters.")
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

    //
    let ammo_priming_pack = Pack::builder(uuid!("eddfd7b7-3476-4ad7-9302-5cfe77ee4ea6"), "Ammo Priming Pack",  "Contains 2 of each Uncommon ammo booster, plus 2 additional boosters, at least 1 of which is Rare or better.")
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

    //
    let technical_mods_pack = Pack::builder(uuid!("975f87f5-0242-4c73-9e0f-6e4033b22ee9"), "Technical Mods Pack", "Contains 5 random consumables or weapon mods, including at least 1 Uncommon, with a small chance for a Rare.")
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

    //
    let advanced_pack = Pack::builder(uuid!("974a8c8e-08bc-4fdb-bede-43337c255df8"), "Advanced Pack", "Contains 5 random items or characters, including at least 1 Uncommon, with a small chance for a Rare")
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

    //
    let expert_pack = Pack::builder(uuid!("b6fe6a9f-de70-463a-bcc5-a1b146067470"), "Expert Pack", "Contains 5 random items or characters, including at least 1 Rare, with a small chance for an Ultra-Rare")
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

    //
    let reserves_pack = Pack::builder(uuid!("731b16c9-3a97-4166-a2f7-e79c8b45128a"), "Reserves Pack", "Contains 5 random items or characters, including at least 2 that are Rare or better, with a higher chance for characters.")
        .add(
            PackCollection::new(
                items_and_characters_filter
                    .clone()
                    // Apply additional weight to characters
                    .merge(Filter::base_category(BaseCategory::Characters).weight(32))
                    // Exclude Rare and Ultra rare from this selection
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

    //
    let arsenal_pack = Pack::builder(uuid!("29c47d42-5830-435b-943f-bf6cf04145e1"), "Arsenal Pack", "Contains 5 random items or characters, including at least 2 that are Rare or better, with a higher chance for weapons.")
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
                    // Exclude Rare and Ultra rare from this selection
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

    //
    let premium_pack = Pack::builder(uuid!("8344cd62-2aed-468d-b155-6ae01f1f2405"), "Premium Pack", "Contains 5 random items or characters, including at least 2 that are Rare, with a higher chance for at least 1 Ultra-Rare")
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

    //
    let jumbo_premium_pack = Pack::builder(uuid!("e3e56e89-b995-475f-8e75-84bf27dc8297"), "Jumbo Premium Pack", "Contains 25 random items or characters, including at least 10 that are Rare, with 5 improved chances for an Ultra-Rare.")
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

    //
    let bonus_reward_pack = |name: ItemName| {
        Pack::builder(name, "Bonus Reward Pack", "Contains 5 random items or characters, including at least 1 Uncommon, with a small chance for a Rare")
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

    let random_mod_pack = |name: ItemName,
                           rarity: ItemRarity,
                           description: &str,
                           contents_description: &str|
     -> Pack {
        Pack::builder(name, description, contents_description)
            .add(PackCollection::new(
                Filter::base_categories([
                    BaseCategory::WeaponMods,
                    BaseCategory::WeaponModsEnhanced,
                ])
                .and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    let random_weapon_pack = |name: ItemName,
                              rarity: ItemRarity,
                              description: &str,
                              contents_description: &str|
     -> Pack {
        Pack::builder(name, description, contents_description)
            .add(PackCollection::new(
                Filter::base_categories([BaseCategory::Weapons, BaseCategory::WeaponsSpecialized])
                    .and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    let random_character_pack = |name: ItemName,
                                 rarity: ItemRarity,
                                 description: &str,
                                 contents_description: &str|
     -> Pack {
        Pack::builder(name, description, contents_description)
            .add(PackCollection::new(
                Filter::base_category(BaseCategory::Characters).and(Filter::Rarity(rarity)),
            ))
            .build()
    };

    // Pack containing a single item
    let item_pack = |name: ItemName, item: ItemName, description: &str| {
        Pack::builder(
            name,
            description,
            format!("Pack containing one '{description}'"),
        )
        .add(PackCollection::named(item))
        .build()
    };

    // Marker for a pack that is not yet implemented
    let todo = |name: ItemName, description: &str| {
        Pack::builder(name, description, "Unknown contents, to be implemented").build()
    };

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
            "Random Weapon Mod Common",
            "Contains 1 Random Common Weapon Mod",
        ),
        random_mod_pack(
            uuid!("44da78e5-8ceb-4684-983e-794329d4a631"),
            ItemRarity::Uncommon,
            "Random Weapon Mod Uncommon",
            "Contains 1 Random Uncommon Weapon Mod",
        ),
        random_mod_pack(
            uuid!("b104645c-ff63-4081-a3c2-669718d7e570"),
            ItemRarity::Rare,
            "Random Weapon Mod Rare",
            "Contains 1 Random Rare Weapon Mod",
        ),
        // Random weapons
        random_weapon_pack(
            uuid!("20a2212b-ac19-436f-93c9-143463a813e9"),
            ItemRarity::Uncommon,
            "Random Weapon Uncommon",
            "Contains 1 Random Uncommon Weapon",
        ),
        random_weapon_pack(
            uuid!("aea28dd4-b5be-4994-80ec-825e2b024d4d"),
            ItemRarity::Rare,
            "Random Weapon Rare",
            "Contains 1 Random Rare Weapon",
        ),
        random_weapon_pack(
            uuid!("e9bfb771-5244-4f33-b318-dd49d79c7edf"),
            ItemRarity::UltraRare,
            "Random Weapon Ultra Rare",
            "Contains 1 Random Ultra Rare Weapon",
        ),
        // Random characters
        random_character_pack(
            uuid!("e71d0c00-44f2-4087-a7f7-7a138fbee0e9"),
            ItemRarity::Uncommon,
            "Random Character Uncommon",
            "Contains 1 Random Uncommon Character",
        ),
        random_character_pack(
            uuid!("53c8b4d7-18bf-4fc3-97cd-2a8366140b0a"),
            ItemRarity::Rare,
            "Random Character Rare",
            "Contains 1 Random Rare Character",
        ),
        random_character_pack(
            uuid!("dad9ad62-1f36-4e38-9634-2eda92a83096"),
            ItemRarity::UltraRare,
            "Random Character Ultra Rare",
            "Contains 1 Random Ultra Rare Character",
        ),
        // Single item packs
        item_pack(
            uuid!("ff6affa2-226b-4c8b-8013-7e7e94335e88"),
            uuid!("eaefec2a-d892-498b-a175-e5d2048ae39a"),
            "COBRA RPG",
        ),
        item_pack(
            uuid!("784e1293-4480-4abd-965e-2c6584f550c8"),
            uuid!("af39be6b-0542-4997-b524-227aa41ae2eb"),
            "REVIVE PACK",
        ),
        item_pack(
            uuid!("16cdf51b-443a-48e2-ad07-413a3f4370e7"),
            uuid!("2cc0d932-8e9d-48a6-a6e8-a5665b77e835"),
            "AMMO PACK",
        ),
        item_pack(
            uuid!("bc012022-2d42-48d1-88fa-2d905d83d4fd"),
            uuid!("52a2e172-2ae6-49f4-9914-bf3094f3a363"),
            "CHARACTER RESPEC",
        ),
        item_pack(
            uuid!("3a7a1d97-ddb7-4954-85e8-b280c2b9b2dc"),
            uuid!("83d69f5b-3f97-4d41-ad76-99ea37a35ba8"),
            "EXPERIENCE ENHANCER III",
        ),
        item_pack(
            uuid!("a26534c9-636c-4022-8d7e-3f76af5fde02"),
            uuid!("4f46229e-51cd-4ece-9a21-731133348088"),
            "EXPERIENCE ENHANCER II",
        ),
        item_pack(
            uuid!("34a78027-ac6e-4bc6-856e-4b8cee5859be"),
            uuid!("4d790010-1a79-4bd0-a79b-d52cac068a3a"),
            "FIRST AID PACK",
        ),
        todo(uuid!("80a9babf-3088-4ce9-a986-804f6ce9660c"), "APEX PACK"),
        todo(uuid!("3b2c8ed8-df9a-4659-aeda-786e06cc7dd9"), "APEX POINTS"),
        todo(
            uuid!("47088308-e623-494e-a436-cccfd7f4150f"),
            "LOYALTY PACK (ME3)",
        ),
        todo(
            uuid!("523226d2-8a17-4081-9c22-71c890d1b4ab"),
            "LOYALTY PACK (DA:I)",
        ),
        todo(
            uuid!("ab939baf-3cc0-46a8-8983-5c8e92754a25"),
            "BONUS REWARD PACK",
        ),
        todo(
            uuid!("aa7b57df-d0a7-4275-8623-38575565fe15"),
            "PRE-ORDER BOOSTER PACK",
        ),
        todo(
            uuid!("9dba3f79-7c9f-4526-96f0-7eaec177eccf"),
            "ANDROMEDA INITIATIVE PACK",
        ),
        todo(
            uuid!("51e008c4-018c-477e-b99a-e8b44a86483b"),
            "SUPER DELUXE EDITION PACK - 1/20",
        ),
        todo(
            uuid!("80304bc9-e704-4b5d-9193-e35f8de7b871"),
            "SUPER DELUXE EDITION PACK - 2/20",
        ),
        todo(
            uuid!("efcc43cf-5877-4ef4-a52b-c35a88a154d2"),
            "SUPER DELUXE EDITION PACK - 3/20",
        ),
        todo(
            uuid!("3ff3ff1b-d2f1-4912-9612-9c50cf7138e2"),
            "SUPER DELUXE EDITION PACK - 4/20",
        ),
        todo(
            uuid!("22a72362-620c-4c86-bf83-83848336a6fb"),
            "SUPER DELUXE EDITION PACK - 5/20",
        ),
        todo(
            uuid!("66e5a516-443c-4062-953c-d34ffec0e4c5"),
            "SUPER DELUXE EDITION PACK - 6/20",
        ),
        todo(
            uuid!("06a249fd-324d-4a9e-9f46-7cb7e620652d"),
            "SUPER DELUXE EDITION PACK - 7/20",
        ),
        todo(
            uuid!("384e4424-0421-4793-b713-13d68616505e"),
            "SUPER DELUXE EDITION PACK - 8/20",
        ),
        todo(
            uuid!("e78760b4-2c64-45be-9906-e3183c64a424"),
            "SUPER DELUXE EDITION PACK - 9/20",
        ),
        todo(
            uuid!("5baa0a3d-86e3-45cc-8ab1-d26591c46a3c"),
            "SUPER DELUXE EDITION PACK - 10/20",
        ),
        todo(
            uuid!("03d7ec5a-d729-4fb3-91d2-2db11f8dfa40"),
            "SUPER DELUXE EDITION PACK - 11/20",
        ),
        //
        todo(
            uuid!("bed2b13e-1cca-4981-b81f-985c051565a4"),
            "SUPER DELUXE EDITION PACK - 12/20",
        ),
        todo(
            uuid!("d21b1767-cb37-4bfa-ad30-12a9d2240775"),
            "SUPER DELUXE EDITION PACK - 13/20",
        ),
        todo(
            uuid!("cbe39480-8473-4aa4-8a06-ce1524a5af2e"),
            "SUPER DELUXE EDITION PACK - 14/20",
        ),
        todo(
            uuid!("317d54fd-0596-44ea-84ee-30b5fec1ab1d"),
            "SUPER DELUXE EDITION PACK - 15/20",
        ),
        todo(
            uuid!("db74221c-1e7e-41af-9a20-cb8176d5d00b"),
            "SUPER DELUXE EDITION PACK - 16/20",
        ),
        todo(
            uuid!("c1a96446-ae8e-47f5-8770-caeb69f862bd"),
            "SUPER DELUXE EDITION PACK - 17/20",
        ),
        todo(
            uuid!("774be722-7814-4c72-9d6f-08e5bf98aa47"),
            "SUPER DELUXE EDITION PACK - 18/20",
        ),
        todo(
            uuid!("b0fce148-f9d8-4098-b767-0e3e523f6e0d"),
            "SUPER DELUXE EDITION PACK - 19/20",
        ),
        todo(
            uuid!("23f98283-f960-46d6-85f9-4bf85d60e2cd"),
            "SUPER DELUXE EDITION PACK - 20/20",
        ),
        todo(
            uuid!("c4b1ebe3-e0b0-42fb-a51c-c6c2d688ac71"),
            "APEX REINFORCEMENT PACK",
        ),
        todo(
            uuid!("203ce2dc-962f-44c8-a513-76ee2286d0b7"),
            "APEX COMMENDATION PACK",
        ),
        todo(
            uuid!("17f90be7-8d74-4593-a85f-0b4cdb9f57ba"),
            "APEX CHALLENGE PACK",
        ),
        todo(
            uuid!("7f2a365a-9f08-412f-8490-ce55fd34aad6"),
            "LOGITECH WEAPON PACK",
        ),
        todo(
            uuid!("33cb8ec3-efce-4744-a858-db5e60e11424"),
            "BONUS BOOSTER PACK",
        ),
        todo(
            uuid!("fcc1fbf1-fa53-445b-b2e9-561702795627"),
            "SUPPORT PACK",
        ),
        todo(
            uuid!("d8b62c9a-31f2-4e7e-82fe-43b9e72cbc7f"),
            "TOTINO'S BOOSTER PACK",
        ),
        todo(
            uuid!("8a072bab-e849-475d-b552-e18704b150c4"),
            "APEX HQ PACK",
        ),
        todo(
            uuid!("6fcbb0d5-b4ed-406d-8056-029ce7a91fd0"),
            "ADVANCED COMMUNITY PACK",
        ),
        todo(
            uuid!("cba5b757-cf67-40e1-a500-66dad3840088"),
            "STARTER PACK",
        ),
        todo(
            uuid!("37101bb8-e5c0-44d7-bcd9-bf49ceecc1de"),
            "TUTORIAL PACK",
        ),
        todo(
            uuid!("cc15e17f-1b06-4413-9c6c-544d01b50f2a"),
            "DELUXE EDITION PACK",
        ),
        item_pack(
            uuid!("208aa537-19d0-4bea-9ac9-f11713cd85e8"),
            uuid!("dd241aa0-26ba-4165-8332-69ba6259a8d3"),
            "NAMEPLATE: APEX MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("c9334ea7-9249-46a7-93af-b0622af5370e"),
            uuid!("ec666f35-cc51-4569-87ca-3c17ff25efe4"),
            "NAMEPLATE: APEX MASTERY - SILVER",
        ),
        item_pack(
            uuid!("7ad4c7ea-2b31-412a-b688-c2d56619dcc3"),
            uuid!("dec5e82a-0151-4802-b9eb-064e1849cba1"),
            "NAMEPLATE: APEX MASTERY - GOLD",
        ),
        item_pack(
            uuid!("0b7386e1-3e9b-415e-b246-45d3674367f4"),
            uuid!("bcec3018-405b-4c52-86b5-d4aedacccbd7"),
            "NAMEPLATE: ASSAULT RIFLE MASTERY- BRONZE",
        ),
        item_pack(
            uuid!("0d31bf4b-3ab2-4d09-8028-335bb2f28ad8"),
            uuid!("fdd1d812-64e1-40e9-ad89-3b7f90641fab"),
            "NAMEPLATE: ASSAULT RIFLE MASTERY- SILVER",
        ),
        item_pack(
            uuid!("19a680d4-5149-420a-aebe-03b9beb1ab83"),
            uuid!("1fa00e66-177d-4afb-831c-ca90fcf09e91"),
            "NAMEPLATE: ASSAULT RIFLE MASTERY- GOLD",
        ),
        item_pack(
            uuid!("d7e1823e-aa41-47fe-9602-13b6f31153f6"),
            uuid!("34a56ba9-1e06-4b27-8fb5-ca8122c6ac72"),
            "NAMEPLATE: COMBAT MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("5d3d4ce8-9cf0-4ff6-9860-9e8554c10577"),
            uuid!("429c1c96-1aa6-4b9a-a109-754d4f1ce3ab"),
            "NAMEPLATE: COMBAT MASTERY - SILVER",
        ),
        item_pack(
            uuid!("c537155c-efbd-49c2-a15c-2fcd088dfeb2"),
            uuid!("f958a50a-f9d4-477c-b071-d278fe6fa581"),
            "NAMEPLATE: COMBAT MASTERY - GOLD",
        ),
        item_pack(
            uuid!("f8a12dd0-dd4d-4151-91dc-7e019005a22c"),
            uuid!("26a31baf-8fef-4e8f-b704-29e9f335df0e"),
            "NAMEPLATE: KETT MASTERY- BRONZE",
        ),
        item_pack(
            uuid!("e1c4ff7d-63e5-4e82-ae89-a078b954edce"),
            uuid!("1d832caf-8ed5-4329-b33d-06d0ad9463f4"),
            "NAMEPLATE: KETT MASTERY- SILVER",
        ),
        item_pack(
            uuid!("65e537a8-0a56-4ded-8d48-41e68d9d82cb"),
            uuid!("4d9c88f4-22d6-4096-8d5a-3e6629adf34f"),
            "NAMEPLATE: KETT MASTERY- GOLD",
        ),
        item_pack(
            uuid!("3dbc20f9-4258-44c8-aace-f89444f48346"),
            uuid!("59cbef6f-323b-47c2-93e1-a41bdef50d14"),
            "NAMEPLATE: MAP MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("6d05ac99-3e2e-4f48-9b84-04c8d9be8420"),
            uuid!("8a3fbe71-eced-4d03-8cdc-f8ba3888b53c"),
            "NAMEPLATE: MAP MASTERY - SILVER",
        ),
        item_pack(
            uuid!("ba606bb6-08b0-4002-b45e-ab0d07c4126d"),
            uuid!("129c6111-fdb8-4907-a820-8f9665de6d80"),
            "NAMEPLATE: MAP MASTERY - GOLD",
        ),
        item_pack(
            uuid!("ce59f903-f3a1-4ec3-90a3-1e82c5f47b85"),
            uuid!("c2dd50c5-d650-4a75-bd49-f476a4e9d18e"),
            "NAMEPLATE: OUTLAW MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("2d9e2f93-2c72-491e-bdb9-46f20d0d9339"),
            uuid!("713b03ba-cead-4cd7-8239-0ce38dbc32fb"),
            "NAMEPLATE: OUTLAW MASTERY - SILVER",
        ),
        item_pack(
            uuid!("daf74c9a-8c2b-4de4-931f-dce265a88c1c"),
            uuid!("9223bffe-ce83-48bf-8eb5-ed9e7345bdaa"),
            "NAMEPLATE: OUTLAW MASTERY - GOLD",
        ),
        item_pack(
            uuid!("5c7b9f32-4fef-430c-a72d-0e7409b84adc"),
            uuid!("80c863cc-d53f-4335-92bd-71d6cec3b08b"),
            "NAMEPLATE: APEX RATING - BRONZE",
        ),
        item_pack(
            uuid!("ad9c5a2f-63b0-4638-935c-1733f083de38"),
            uuid!("227809cc-1fdd-433a-83ea-0662778e36dd"),
            "NAMEPLATE: APEX RATING - SILVER",
        ),
        item_pack(
            uuid!("74f437e4-fd7d-4f6a-a441-66e6c64bb3c5"),
            uuid!("07a2c3ed-269a-46a4-ab81-5aaa3ff586d8"),
            "NAMEPLATE: APEX RATING - GOLD",
        ),
        item_pack(
            uuid!("414b173e-2dcf-4587-8cdd-43c5bc872c5c"),
            uuid!("5fda99e2-93aa-4e62-a198-c1a4381d9b97"),
            "NAMEPLATE: PISTOL MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("be469a8c-71d0-47f2-a13f-80c94beec052"),
            uuid!("23511ee2-1a01-4d4d-94ef-618a3c199b2b"),
            "NAMEPLATE: PISTOL MASTERY - SILVER",
        ),
        item_pack(
            uuid!("73564b68-8e80-48b1-881c-2e2085787509"),
            uuid!("3164389f-46aa-4f10-b5cb-4c5839a00f57"),
            "NAMEPLATE: PISTOL MASTERY - GOLD",
        ),
        item_pack(
            uuid!("a6248be2-1647-4e9b-9e1e-b8b69ecf809d"),
            uuid!("561289b5-9efa-4d6f-acf4-ce8c2ff26792"),
            "NAMEPLATE: REMNANT MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("123b3fa1-565e-456f-b08d-aa131b0c5cf1"),
            uuid!("4006a2e7-c0b5-4d02-b542-1c14ea05e9a4"),
            "NAMEPLATE: REMNANT MASTERY - SILVER",
        ),
        item_pack(
            uuid!("206115c9-c953-4ce2-aab0-6804660f6cc1"),
            uuid!("9f571cb9-3846-41a0-a0c9-abc7dfac2772"),
            "NAMEPLATE: REMNANT MASTERY - GOLD",
        ),
        item_pack(
            uuid!("aa7b4129-1e67-421a-a3e9-27813bd1105a"),
            uuid!("771029a8-e7ed-46a5-af30-e87ee73350f1"),
            "NAMEPLATE: SHOTGUN MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("88a7e312-1591-4ac5-bdd8-6be1a6f02c9f"),
            uuid!("bed37817-170d-4144-9434-3ccd58c7ec8f"),
            "NAMEPLATE: SHOTGUN MASTERY - SILVER",
        ),
        item_pack(
            uuid!("fa6aab20-ae9a-4778-829b-978f075de939"),
            uuid!("4fa9a564-dfbd-4c28-8ba5-6e9e3e48d950"),
            "NAMEPLATE: SHOTGUN MASTERY - GOLD",
        ),
        item_pack(
            uuid!("66e865bb-b694-4f2a-86e3-caf58442780d"),
            uuid!("2e0c84a8-0495-469e-a059-b71759cadf0a"),
            "NAMEPLATE: SNIPER RIFLE MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("254dad07-4f5b-4ce0-9d78-6be17855f082"),
            uuid!("9945b0d6-2515-4329-a718-cfe1fb26b2d0"),
            "NAMEPLATE: SNIPER RIFLE MASTERY - SILVER",
        ),
        item_pack(
            uuid!("d9e0d08d-5ffc-4e33-9509-40776591eb68"),
            uuid!("6282e95d-5b15-482d-96bc-060e34126177"),
            "NAMEPLATE: SNIPER RIFLE MASTERY - GOLD",
        ),
        item_pack(
            uuid!("6d830d65-13de-4c70-8fb9-d076c569b4f0"),
            uuid!("153c87ec-0b2f-4cc1-9a84-4ad646d1418f"),
            "NAMEPLATE: TECH MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("8fd74763-e397-45ab-a27a-ac8f08e062e1"),
            uuid!("beefc0ed-d91c-463e-bc2c-ade1c9927ab5"),
            "NAMEPLATE: TECH MASTERY - SILVER",
        ),
        item_pack(
            uuid!("737be245-d4ae-410b-9bf8-3db805eb79b7"),
            uuid!("6dbd41ae-c394-4502-984b-228075eada9f"),
            "NAMEPLATE: TECH MASTERY - GOLD",
        ),
        item_pack(
            uuid!("6b1179d1-0a7b-496c-83e2-f66de8b57736"),
            uuid!("70f12a9a-a979-4d62-bda1-5f161e8f133a"),
            "NAMEPLATE: BIOTIC MASTERY - BRONZE",
        ),
        item_pack(
            uuid!("e9d39579-0f21-4d35-952f-cd418b6c4b57"),
            uuid!("9288bbdb-c045-439c-8771-651b83c294cc"),
            "NAMEPLATE: BIOTIC MASTERY - SILVER",
        ),
        item_pack(
            uuid!("8b9263f0-a660-48b3-8a83-f11cfb4da11b"),
            uuid!("c072a185-7173-4a4b-87ce-c76e2ac9cead"),
            "NAMEPLATE: BIOTIC MASTERY - GOLD",
        ),
        todo(uuid!("53a5fc5e-3ba9-476f-a537-555bac6014f3"), "AESTHETIC"),
        todo(uuid!("8425ccb0-37f4-4d5e-915c-0806602f2593"), "AESTHETIC"),
        todo(uuid!("361895d8-49b0-4d0c-b359-60e7c343f194"), "AESTHETIC"),
        todo(uuid!("1e6627c8-f8ee-4c70-86b2-0c2dd4c65ff4"), "AESTHETIC"),
        todo(uuid!("c869e5a6-cb6c-4580-a162-d5ac3f72b737"), "AESTHETIC"),
        todo(uuid!("6e67e5e2-89c7-44cc-89fb-432e8e99734a"), "AESTHETIC"),
        todo(uuid!("55d1d22f-0ee7-41bf-939a-0aa372bb2e72"), "AESTHETIC"),
        todo(uuid!("e3f10da1-312a-4ba4-ad33-0c503e6c2a8f"), "AESTHETIC"),
        todo(uuid!("c9d603e7-9e20-4d72-a672-81c1a188a320"), "AESTHETIC"),
        todo(
            uuid!("e57690fe-4b17-4b11-b1de-a1fd4b0b4a55"),
            "DELUXE EDITION PACK #2",
        ),
        todo(
            uuid!("77459eda-2eab-4aae-b8f0-d26964f269eb"),
            "EA ACCESS PACK",
        ),
        todo(
            uuid!("e28207db-3b14-4ba7-9dc6-d0826d76b78d"),
            "TECH TEST SIGN-UP - BRONZE",
        ),
        todo(
            uuid!("7c4118cd-53fa-4c15-951c-6c250549db1d"),
            "ORIGIN ACCESS PACK",
        ),
        todo(
            uuid!("0d9a69e0-cad5-4242-8052-9f0c2ded0236"),
            "SUPPORT PACK",
        ),
        todo(
            uuid!("5e7cf499-4f72-47d8-b87b-04162ef4e406"),
            "APEX ELITE PACK",
        ),
        //
        todo(
            uuid!("0b2986da-3d0d-45fd-b0b7-2adfca9d2994"),
            "MEA DEVELOPER - GOLD",
        ),
        //
        todo(
            uuid!("a883a017-1b11-41ea-b98a-127b25dd3032"),
            "CELEBRATORY PACK",
        ),
        todo(
            uuid!("5aebef08-b14c-40df-95fe-59fc78274ad5"),
            "CELEBRATORY PACK",
        ),
        todo(
            uuid!("eed5b4df-736d-4b4c-b683-96c19dc5088d"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("eb4fe1a6-c942-43f9-91f5-7b981ccbbb55"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("ccb3f225-e808-4057-99b8-48a33c966be1"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("ef8d85dc-74c5-4554-86c2-4e2f5c7e0fb8"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("f1473ab2-55c1-4b22-a8d2-344dba5b4e09"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("43eed42a-643a-4ddc-b0b7-51e6ed5ccbf8"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("67416130-bd36-4cf4-94df-e276f7642472"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("a1e73511-3672-40b0-9a9f-8c24faa8b831"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("23b6647a-0b54-43a8-85fb-0a382522bf97"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("609be685-d3c3-43a6-b0a1-484701c19172"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("e4e12a1d-6f0a-4191-a740-26e715e42abe"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("f8aecee2-3add-4b73-a520-961ef9932ea2"),
            "MP DLC PACK - COLLECTION ITEMS",
        ),
        todo(
            uuid!("694577c3-0d92-4e85-ad41-de54a4c91154"),
            "[BUG] I am a banner!",
        ),
    ]
    .into_iter()
    .map(|pack| (pack.name, pack))
    .collect()
}

#[cfg(test)]
mod test {
    use super::generate_packs;
    use crate::definitions::packs::Pack;
    use std::collections::HashMap;

    /// Tests that the packs JSON generated from code matches that which is
    /// serialized and parsed back again
    #[test]
    fn test_generated_packs_matches() {
        let packs = generate_packs();
        let value = serde_json::to_string(&packs).unwrap();
        let parsed: HashMap<uuid::Uuid, Pack> = serde_json::from_str(&value).unwrap();

        let mut left = packs.into_iter().collect::<Vec<_>>();
        let mut right = parsed.into_iter().collect::<Vec<_>>();

        left.sort_by_key(|value| value.0);
        right.sort_by_key(|value| value.0);

        assert_eq!(left, right);
    }

    /// Generates a packs file from the current data
    #[test]
    #[ignore = "not an automated test, utility for generating a file"]
    fn generate_packs_file() {
        let packs = generate_packs();
        let value = serde_json::to_string_pretty(&packs).unwrap();
        std::fs::write("packs.json", value).unwrap();
    }
}
