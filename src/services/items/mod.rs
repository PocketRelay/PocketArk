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

pub mod pack;
pub mod v2;

/// Type of the name for items, names are [Uuid]s with some exceptions (Thanks EA)
pub type ItemName = Uuid;

pub struct ItemsService {
    /// Item definitions
    defs: Vec<ItemDefinition>,
    /// Lookup table for item definitions based on their name
    defs_by_name: HashMap<Uuid, usize>,
    /// Available unlock packs
    packs: HashMap<Uuid, Pack>,
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
            Self::bonus_reward_pack("cf9cd252-e1f2-4574-973d-d66cd81558d3"),
            Self::bonus_reward_pack("ab939baf-3cc0-46a8-8983-5c8e92754a25"),
            // Random mods
            Self::random_mod_pack("890b2aa6-191f-4162-ae79-a78d23e3c505", Rarity::COMMON),
            Self::random_mod_pack("44da78e5-8ceb-4684-983e-794329d4a631", Rarity::UNCOMMON),
            Self::random_mod_pack("b104645c-ff63-4081-a3c2-669718d7e570", Rarity::RARE),
            // Random weapons
            Self::random_weapon_pack("20a2212b-ac19-436f-93c9-143463a813e9", Rarity::UNCOMMON),
            Self::random_weapon_pack("aea28dd4-b5be-4994-80ec-825e2b024d4d", Rarity::RARE),
            Self::random_weapon_pack("e9bfb771-5244-4f33-b318-dd49d79c7edf", Rarity::ULTRA_RARE),
            // Random characters
            Self::random_character_pack("e71d0c00-44f2-4087-a7f7-7a138fbee0e9", Rarity::UNCOMMON),
            Self::random_character_pack("53c8b4d7-18bf-4fc3-97cd-2a8366140b0a", Rarity::RARE),
            Self::random_character_pack("dad9ad62-1f36-4e38-9634-2eda92a83096", Rarity::ULTRA_RARE),
            // Single item packs

            // COBRA RPG
            Self::item_pack(
                "ff6affa2-226b-4c8b-8013-7e7e94335e88",
                "eaefec2a-d892-498b-a175-e5d2048ae39a",
            ),
            // REVIVE PACK
            Self::item_pack(
                "784e1293-4480-4abd-965e-2c6584f550c8",
                "af39be6b-0542-4997-b524-227aa41ae2eb",
            ),
            // AMMO PACK
            Self::item_pack(
                "16cdf51b-443a-48e2-ad07-413a3f4370e7",
                "2cc0d932-8e9d-48a6-a6e8-a5665b77e835",
            ),
            // CHARACTER RESPEC
            Self::item_pack(
                "bc012022-2d42-48d1-88fa-2d905d83d4fd",
                "52a2e172-2ae6-49f4-9914-bf3094f3a363",
            ),
            // EXPERIENCE ENHANCER III
            Self::item_pack(
                "3a7a1d97-ddb7-4954-85e8-b280c2b9b2dc",
                "83d69f5b-3f97-4d41-ad76-99ea37a35ba8",
            ),
            // EXPERIENCE ENHANCER II
            Self::item_pack(
                "a26534c9-636c-4022-8d7e-3f76af5fde02",
                "4f46229e-51cd-4ece-9a21-731133348088",
            ),
            // FIRST AID PACK
            Self::item_pack(
                "34a78027-ac6e-4bc6-856e-4b8cee5859be",
                "4d790010-1a79-4bd0-a79b-d52cac068a3a",
            ),
            // APEX PACK
            Self::todo("80a9babf-3088-4ce9-a986-804f6ce9660c"),
            // APEX POINTS
            Self::todo("3b2c8ed8-df9a-4659-aeda-786e06cc7dd9"),
            // LOYALTY PACK (ME3)
            Self::todo("47088308-e623-494e-a436-cccfd7f4150f"),
            // LOYALTY PACK (DA:I)
            Self::todo("523226d2-8a17-4081-9c22-71c890d1b4ab"),
            // BONUS REWARD PACK
            Self::todo("ab939baf-3cc0-46a8-8983-5c8e92754a25"),
            // PRE-ORDER BOOSTER PACK
            Self::todo("aa7b57df-d0a7-4275-8623-38575565fe15"),
            // ANDROMEDA INITIATIVE PACK
            Self::todo("9dba3f79-7c9f-4526-96f0-7eaec177eccf"),
            // SUPER DELUXE EDITION PACK - 1/20
            Self::todo("51e008c4-018c-477e-b99a-e8b44a86483b"),
            // SUPER DELUXE EDITION PACK - 2/20
            Self::todo("80304bc9-e704-4b5d-9193-e35f8de7b871"),
            // SUPER DELUXE EDITION PACK - 3/20
            Self::todo("efcc43cf-5877-4ef4-a52b-c35a88a154d2"),
            // SUPER DELUXE EDITION PACK - 4/20
            Self::todo("3ff3ff1b-d2f1-4912-9612-9c50cf7138e2"),
            // SUPER DELUXE EDITION PACK - 5/20
            Self::todo("22a72362-620c-4c86-bf83-83848336a6fb"),
            // SUPER DELUXE EDITION PACK - 6/20
            Self::todo("66e5a516-443c-4062-953c-d34ffec0e4c5"),
            // SUPER DELUXE EDITION PACK - 7/20
            Self::todo("06a249fd-324d-4a9e-9f46-7cb7e620652d"),
            // SUPER DELUXE EDITION PACK - 8/20
            Self::todo("384e4424-0421-4793-b713-13d68616505e"),
            // SUPER DELUXE EDITION PACK - 9/20
            Self::todo("e78760b4-2c64-45be-9906-e3183c64a424"),
            // SUPER DELUXE EDITION PACK - 10/20
            Self::todo("5baa0a3d-86e3-45cc-8ab1-d26591c46a3c"),
            // SUPER DELUXE EDITION PACK - 11/20
            Self::todo("03d7ec5a-d729-4fb3-91d2-2db11f8dfa40"),
            // SUPER DELUXE EDITION PACK - 12/20
            Self::todo("bed2b13e-1cca-4981-b81f-985c051565a4"),
            // SUPER DELUXE EDITION PACK - 13/20
            Self::todo("d21b1767-cb37-4bfa-ad30-12a9d2240775"),
            // SUPER DELUXE EDITION PACK - 14/20
            Self::todo("cbe39480-8473-4aa4-8a06-ce1524a5af2e"),
            // SUPER DELUXE EDITION PACK - 15/20
            Self::todo("317d54fd-0596-44ea-84ee-30b5fec1ab1d"),
            // SUPER DELUXE EDITION PACK - 16/20
            Self::todo("db74221c-1e7e-41af-9a20-cb8176d5d00b"),
            // SUPER DELUXE EDITION PACK - 17/20
            Self::todo("c1a96446-ae8e-47f5-8770-caeb69f862bd"),
            // SUPER DELUXE EDITION PACK - 18/20
            Self::todo("774be722-7814-4c72-9d6f-08e5bf98aa47"),
            // SUPER DELUXE EDITION PACK - 19/20
            Self::todo("b0fce148-f9d8-4098-b767-0e3e523f6e0d"),
            // SUPER DELUXE EDITION PACK - 20/20
            Self::todo("23f98283-f960-46d6-85f9-4bf85d60e2cd"),
            // APEX REINFORCEMENT PACK
            Self::todo("c4b1ebe3-e0b0-42fb-a51c-c6c2d688ac71"),
            // APEX COMMENDATION PACK
            Self::todo("203ce2dc-962f-44c8-a513-76ee2286d0b7"),
            // APEX CHALLENGE PACK
            Self::todo("17f90be7-8d74-4593-a85f-0b4cdb9f57ba"),
            // LOGITECH WEAPON PACK
            Self::todo("7f2a365a-9f08-412f-8490-ce55fd34aad6"),
            // BONUS BOOSTER PACK
            Self::todo("33cb8ec3-efce-4744-a858-db5e60e11424"),
            // SUPPORT PACK
            Self::todo("fcc1fbf1-fa53-445b-b2e9-561702795627"),
            // TOTINO'S BOOSTER PACK
            Self::todo("d8b62c9a-31f2-4e7e-82fe-43b9e72cbc7f"),
            // APEX HQ PACK
            Self::todo("8a072bab-e849-475d-b552-e18704b150c4"),
            // ADVANCED COMMUNITY PACK
            Self::todo("6fcbb0d5-b4ed-406d-8056-029ce7a91fd0"),
            // STARTER PACK
            Self::todo("cba5b757-cf67-40e1-a500-66dad3840088"),
            // TUTORIAL PACK
            Self::todo("37101bb8-e5c0-44d7-bcd9-bf49ceecc1de"),
            // DELUXE EDITION PACK
            Self::todo("cc15e17f-1b06-4413-9c6c-544d01b50f2a"),
            // NAMEPLATE: APEX MASTERY - BRONZE
            Self::item_pack(
                "208aa537-19d0-4bea-9ac9-f11713cd85e8",
                "dd241aa0-26ba-4165-8332-69ba6259a8d3",
            ),
            // NAMEPLATE: APEX MASTERY - SILVER
            Self::item_pack(
                "c9334ea7-9249-46a7-93af-b0622af5370e",
                "ec666f35-cc51-4569-87ca-3c17ff25efe4",
            ),
            // NAMEPLATE: APEX MASTERY - GOLD
            Self::item_pack(
                "7ad4c7ea-2b31-412a-b688-c2d56619dcc3",
                "dec5e82a-0151-4802-b9eb-064e1849cba1",
            ),
            // NAMEPLATE: ASSAULT RIFLE MASTERY- BRONZE
            Self::item_pack(
                "0b7386e1-3e9b-415e-b246-45d3674367f4",
                "bcec3018-405b-4c52-86b5-d4aedacccbd7",
            ),
            // NAMEPLATE: ASSAULT RIFLE MASTERY- SILVER
            Self::item_pack(
                "0d31bf4b-3ab2-4d09-8028-335bb2f28ad8",
                "fdd1d812-64e1-40e9-ad89-3b7f90641fab",
            ),
            // NAMEPLATE: ASSAULT RIFLE MASTERY- GOLD
            Self::item_pack(
                "19a680d4-5149-420a-aebe-03b9beb1ab83",
                "1fa00e66-177d-4afb-831c-ca90fcf09e91",
            ),
            // NAMEPLATE: COMBAT MASTERY - BRONZE
            Self::item_pack(
                "d7e1823e-aa41-47fe-9602-13b6f31153f6",
                "34a56ba9-1e06-4b27-8fb5-ca8122c6ac72",
            ),
            // NAMEPLATE: COMBAT MASTERY - SILVER
            Self::item_pack(
                "5d3d4ce8-9cf0-4ff6-9860-9e8554c10577",
                "429c1c96-1aa6-4b9a-a109-754d4f1ce3ab",
            ),
            // NAMEPLATE: COMBAT MASTERY - GOLD
            Self::item_pack(
                "c537155c-efbd-49c2-a15c-2fcd088dfeb2",
                "f958a50a-f9d4-477c-b071-d278fe6fa581",
            ),
            // NAMEPLATE: KETT MASTERY- BRONZE
            Self::item_pack(
                "f8a12dd0-dd4d-4151-91dc-7e019005a22c",
                "26a31baf-8fef-4e8f-b704-29e9f335df0e",
            ),
            // NAMEPLATE: KETT MASTERY- SILVER
            Self::item_pack(
                "e1c4ff7d-63e5-4e82-ae89-a078b954edce",
                "1d832caf-8ed5-4329-b33d-06d0ad9463f4",
            ),
            // NAMEPLATE: KETT MASTERY- GOLD
            Self::item_pack(
                "65e537a8-0a56-4ded-8d48-41e68d9d82cb",
                "4d9c88f4-22d6-4096-8d5a-3e6629adf34f",
            ),
            // NAMEPLATE: MAP MASTERY - BRONZE
            Self::item_pack(
                "3dbc20f9-4258-44c8-aace-f89444f48346",
                "59cbef6f-323b-47c2-93e1-a41bdef50d14",
            ),
            // NAMEPLATE: MAP MASTERY - SILVER
            Self::item_pack(
                "6d05ac99-3e2e-4f48-9b84-04c8d9be8420",
                "8a3fbe71-eced-4d03-8cdc-f8ba3888b53c",
            ),
            // NAMEPLATE: MAP MASTERY - GOLD
            Self::item_pack(
                "ba606bb6-08b0-4002-b45e-ab0d07c4126d",
                "129c6111-fdb8-4907-a820-8f9665de6d80",
            ),
            // NAMEPLATE: OUTLAW MASTERY - BRONZE
            Self::item_pack(
                "ce59f903-f3a1-4ec3-90a3-1e82c5f47b85",
                "c2dd50c5-d650-4a75-bd49-f476a4e9d18e",
            ),
            // NAMEPLATE: OUTLAW MASTERY - SILVER
            Self::item_pack(
                "2d9e2f93-2c72-491e-bdb9-46f20d0d9339",
                "713b03ba-cead-4cd7-8239-0ce38dbc32fb",
            ),
            // NAMEPLATE: OUTLAW MASTERY - GOLD
            Self::item_pack(
                "daf74c9a-8c2b-4de4-931f-dce265a88c1c",
                "9223bffe-ce83-48bf-8eb5-ed9e7345bdaa",
            ),
            // NAMEPLATE: APEX RATING - BRONZE
            Self::item_pack(
                "5c7b9f32-4fef-430c-a72d-0e7409b84adc",
                "80c863cc-d53f-4335-92bd-71d6cec3b08b",
            ),
            // NAMEPLATE: APEX RATING - SILVER
            Self::item_pack(
                "ad9c5a2f-63b0-4638-935c-1733f083de38",
                "227809cc-1fdd-433a-83ea-0662778e36dd",
            ),
            // NAMEPLATE: APEX RATING - GOLD
            Self::item_pack(
                "74f437e4-fd7d-4f6a-a441-66e6c64bb3c5",
                "07a2c3ed-269a-46a4-ab81-5aaa3ff586d8",
            ),
            // NAMEPLATE: PISTOL MASTERY - BRONZE
            Self::item_pack(
                "414b173e-2dcf-4587-8cdd-43c5bc872c5c",
                "5fda99e2-93aa-4e62-a198-c1a4381d9b97",
            ),
            // NAMEPLATE: PISTOL MASTERY - SILVER
            Self::item_pack(
                "be469a8c-71d0-47f2-a13f-80c94beec052",
                "23511ee2-1a01-4d4d-94ef-618a3c199b2b",
            ),
            // NAMEPLATE: PISTOL MASTERY - GOLD
            Self::item_pack(
                "73564b68-8e80-48b1-881c-2e2085787509",
                "3164389f-46aa-4f10-b5cb-4c5839a00f57",
            ),
            // NAMEPLATE: REMNANT MASTERY - BRONZE
            Self::item_pack(
                "a6248be2-1647-4e9b-9e1e-b8b69ecf809d",
                "561289b5-9efa-4d6f-acf4-ce8c2ff26792",
            ),
            // NAMEPLATE: REMNANT MASTERY - SILVER
            Self::item_pack(
                "123b3fa1-565e-456f-b08d-aa131b0c5cf1",
                "4006a2e7-c0b5-4d02-b542-1c14ea05e9a4",
            ),
            // NAMEPLATE: REMNANT MASTERY - GOLD
            Self::item_pack(
                "206115c9-c953-4ce2-aab0-6804660f6cc1",
                "9f571cb9-3846-41a0-a0c9-abc7dfac2772",
            ),
            // NAMEPLATE: SHOTGUN MASTERY - BRONZE
            Self::item_pack(
                "aa7b4129-1e67-421a-a3e9-27813bd1105a",
                "771029a8-e7ed-46a5-af30-e87ee73350f1",
            ),
            // NAMEPLATE: SHOTGUN MASTERY - SILVER
            Self::item_pack(
                "88a7e312-1591-4ac5-bdd8-6be1a6f02c9f",
                "bed37817-170d-4144-9434-3ccd58c7ec8f",
            ),
            // NAMEPLATE: SHOTGUN MASTERY - GOLD
            Self::item_pack(
                "fa6aab20-ae9a-4778-829b-978f075de939",
                "4fa9a564-dfbd-4c28-8ba5-6e9e3e48d950",
            ),
            // NAMEPLATE: SNIPER RIFLE MASTERY - BRONZE
            Self::item_pack(
                "66e865bb-b694-4f2a-86e3-caf58442780d",
                "2e0c84a8-0495-469e-a059-b71759cadf0a",
            ),
            // NAMEPLATE: SNIPER RIFLE MASTERY - SILVER
            Self::item_pack(
                "254dad07-4f5b-4ce0-9d78-6be17855f082",
                "9945b0d6-2515-4329-a718-cfe1fb26b2d0",
            ),
            // NAMEPLATE: SNIPER RIFLE MASTERY - GOLD
            Self::item_pack(
                "d9e0d08d-5ffc-4e33-9509-40776591eb68",
                "6282e95d-5b15-482d-96bc-060e34126177",
            ),
            // NAMEPLATE: TECH MASTERY - BRONZE
            Self::item_pack(
                "6d830d65-13de-4c70-8fb9-d076c569b4f0",
                "153c87ec-0b2f-4cc1-9a84-4ad646d1418f",
            ),
            // NAMEPLATE: TECH MASTERY - SILVER
            Self::item_pack(
                "8fd74763-e397-45ab-a27a-ac8f08e062e1",
                "beefc0ed-d91c-463e-bc2c-ade1c9927ab5",
            ),
            // NAMEPLATE: TECH MASTERY - GOLD
            Self::item_pack(
                "737be245-d4ae-410b-9bf8-3db805eb79b7",
                "6dbd41ae-c394-4502-984b-228075eada9f",
            ),
            // NAMEPLATE: BIOTIC MASTERY - BRONZE
            Self::item_pack(
                "6b1179d1-0a7b-496c-83e2-f66de8b57736",
                "70f12a9a-a979-4d62-bda1-5f161e8f133a",
            ),
            // NAMEPLATE: BIOTIC MASTERY - SILVER
            Self::item_pack(
                "e9d39579-0f21-4d35-952f-cd418b6c4b57",
                "9288bbdb-c045-439c-8771-651b83c294cc",
            ),
            // NAMEPLATE: BIOTIC MASTERY - GOLD
            Self::item_pack(
                "8b9263f0-a660-48b3-8a83-f11cfb4da11b",
                "c072a185-7173-4a4b-87ce-c76e2ac9cead",
            ),
            // AESTHETIC
            Self::todo("53a5fc5e-3ba9-476f-a537-555bac6014f3"),
            Self::todo("8425ccb0-37f4-4d5e-915c-0806602f2593"),
            Self::todo("361895d8-49b0-4d0c-b359-60e7c343f194"),
            Self::todo("1e6627c8-f8ee-4c70-86b2-0c2dd4c65ff4"),
            Self::todo("c869e5a6-cb6c-4580-a162-d5ac3f72b737"),
            Self::todo("6e67e5e2-89c7-44cc-89fb-432e8e99734a"),
            Self::todo("55d1d22f-0ee7-41bf-939a-0aa372bb2e72"),
            Self::todo("e3f10da1-312a-4ba4-ad33-0c503e6c2a8f"),
            Self::todo("c9d603e7-9e20-4d72-a672-81c1a188a320"),
            // DELUXE EDITION PACK #2
            Self::todo("e57690fe-4b17-4b11-b1de-a1fd4b0b4a55"),
            // EA ACCESS PACK
            Self::todo("77459eda-2eab-4aae-b8f0-d26964f269eb"),
            // TECH TEST SIGN-UP - BRONZE
            Self::todo("e28207db-3b14-4ba7-9dc6-d0826d76b78d"),
            // ORIGIN ACCESS PACK
            Self::todo("7c4118cd-53fa-4c15-951c-6c250549db1d"),
            // SUPPORT PACK
            Self::todo("0d9a69e0-cad5-4242-8052-9f0c2ded0236"),
            // APEX ELITE PACK
            Self::todo("5e7cf499-4f72-47d8-b87b-04162ef4e406"),
            // MEA DEVELOPER - GOLD
            Self::todo("0b2986da-3d0d-45fd-b0b7-2adfca9d2994"),
            // CELEBRATORY PACK
            Self::todo("a883a017-1b11-41ea-b98a-127b25dd3032"),
            Self::todo("5aebef08-b14c-40df-95fe-59fc78274ad5"),
            // MP DLC PACK - COLLECTION ITEMS
            Self::todo("eed5b4df-736d-4b4c-b683-96c19dc5088d"),
            Self::todo("eb4fe1a6-c942-43f9-91f5-7b981ccbbb55"),
            Self::todo("ccb3f225-e808-4057-99b8-48a33c966be1"),
            Self::todo("ef8d85dc-74c5-4554-86c2-4e2f5c7e0fb8"),
            Self::todo("f1473ab2-55c1-4b22-a8d2-344dba5b4e09"),
            Self::todo("43eed42a-643a-4ddc-b0b7-51e6ed5ccbf8"),
            Self::todo("67416130-bd36-4cf4-94df-e276f7642472"),
            Self::todo("a1e73511-3672-40b0-9a9f-8c24faa8b831"),
            Self::todo("23b6647a-0b54-43a8-85fb-0a382522bf97"),
            Self::todo("609be685-d3c3-43a6-b0a1-484701c19172"),
            Self::todo("e4e12a1d-6f0a-4191-a740-26e715e42abe"),
            Self::todo("f8aecee2-3add-4b73-a520-961ef9932ea2"),
            // [BUG] I am a banner!
            Self::todo("694577c3-0d92-4e85-ad41-de54a4c91154"),
        ]
        .into_iter()
        .map(|pack| (pack.name, pack))
        .collect();

        let packs: HashMap<Uuid, Pack> = HashMap::new();
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

    pub fn pack_by_name(&self, name: &Uuid) -> Option<&Pack> {
        self.packs.get(name)
    }

    /// Single item packs
    fn item_pack(uuid: &str, item: &str) -> Pack {
        Pack::new(uuid).add_item(ItemChance::named(
            Uuid::from_str(item).expect("Invalid item pack ID"),
        ))
    }

    // Pack thats not yet implemented
    fn todo(uuid: &str) -> Pack {
        Pack::new(uuid)
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

    fn bonus_reward_pack(uuid: &str) -> Pack {
        Pack::new(uuid)
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
            .add_item(ItemChance::new(ItemFilter::and(
                // Uncommon but with a chance for rare
                ItemFilter::rarity(Rarity::COMMON).weight(6)
                    | ItemFilter::rarity(Rarity::RARE).weight(1),
                // Items or characters (weighted for weapons)
                ItemFilter::categories(Category::ITEMS_WITH_CHARACTERS),
            )))
    }

    fn random_mod_pack(uuid: &str, rarity: &str) -> Pack {
        Pack::new(uuid).add_item(ItemChance::new(ItemFilter::and(
            ItemFilter::rarity(rarity),
            ItemFilter::category(Category::WEAPON_MODS)
                | ItemFilter::category(Category::WEAPON_MODS_ENHANCED),
        )))
    }

    fn random_weapon_pack(uuid: &str, rarity: &str) -> Pack {
        Pack::new(uuid).add_item(ItemChance::new(ItemFilter::and(
            ItemFilter::rarity(rarity),
            ItemFilter::category(Category::WEAPONS)
                | ItemFilter::category(Category::WEAPONS_SPECIALIZED),
        )))
    }

    fn random_character_pack(uuid: &str, rarity: &str) -> Pack {
        Pack::new(uuid).add_item(ItemChance::new(ItemFilter::and(
            ItemFilter::rarity(rarity),
            ItemFilter::category(Category::CHARACTERS),
        )))
    }
}

pub struct Rarity {}

impl Rarity {
    pub const COMMON: &'static str = "0";
    pub const UNCOMMON: &'static str = "1";
    pub const RARE: &'static str = "2";
    pub const ULTRA_RARE: &'static str = "3";
}

pub struct Category;

#[allow(unused)]
impl Category {
    /// Character items
    pub const CHARACTERS: &'static str = "0";

    // Weapons
    pub const WEAPONS: &'static str = "1:";
    pub const ASSAULT_RIFLE: &'static str = "1:AssaultRifle";
    pub const PISTOL: &'static str = "1:Pistol";
    pub const SHOTGUN: &'static str = "1:Shotgun";
    pub const SNIPER_RIFLE: &'static str = "1:SniperRifle";

    // Weapon mods
    pub const WEAPON_MODS: &'static str = "2:";
    pub const ASSAULT_RIFLE_MODS: &'static str = "2:AssaultRifle";
    pub const PISTOL_MODS: &'static str = "2:Pistol";
    pub const SHOTGUN_MODS: &'static str = "2:Shotgun";
    pub const SNIPER_RIFLE_MODS: &'static str = "2:SniperRifle";

    /// Boosters such as "AMMO CAPACITY MOD I", "ASSAULT RIFLE RAIL AMP", "CRYO AMMO"
    pub const BOOSTERS: &'static str = "3";

    // Consumable items such as "AMMO PACK", "COBTRA RPG", "REVIVE PACK"
    pub const CONSUMABLE: &'static str = "4";

    /// Equipment such as "ADAPTIVE WAR AMP", and "ASSAULT LOADOUT"
    pub const EQUIPMENT: &'static str = "5";

    /// Rewards from challenges
    pub const CHALLENGE_REWARD: &'static str = "7";

    /// Non droppable rewards for apex points
    pub const APEX_POINTS: &'static str = "8";

    /// Upgrades for capacity such as "AMMO PACK CAPACITY INCREASE" and
    /// "CHARACTER RESPEC" items
    pub const CAPACITY_UPGRADE: &'static str = "9";

    /// Rewards from strike team missions (Loot boxes)
    pub const STRIKE_TEAM_REWARD: &'static str = "11";

    /// Item loot box packs /
    pub const ITEM_PACK: &'static str = "12";

    // Specialized gun variants
    pub const WEAPONS_SPECIALIZED: &'static str = "13:";
    pub const ASSAULT_RIFLE_SPECIALIZED: &'static str = "13:AssaultRifle";
    pub const PISTOL_SPECIALIZED: &'static str = "13:Pistol";
    pub const SHOTGUN_SPECIALIZED: &'static str = "13:Shotgun";
    pub const SNIPER_RIFLE_SPECIALIZED: &'static str = "13:SniperRifle";

    // Enhanced weapon mod variants
    pub const WEAPON_MODS_ENHANCED: &'static str = "14:";
    pub const ASSAULT_RIFLE_MODS_ENHANCED: &'static str = "14:AssaultRifle";
    pub const PISTOL_MODS_ENHANCED: &'static str = "14:Pistol";
    pub const SHOTGUN_MODS_ENHANCED: &'static str = "14:Shotgun";
    pub const SNIPER_RIFLE_MODS_ENHANCED: &'static str = "14:SniperRifle";

    pub const ITEMS: &'static [&'static str] = &[
        Self::WEAPONS,
        Self::WEAPON_MODS,
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
        Self::WEAPONS_SPECIALIZED,
        Self::WEAPON_MODS_ENHANCED,
    ];

    pub const ITEMS_NO_WEAPONS: &'static [&'static str] = &[
        Self::BOOSTERS,
        Self::CONSUMABLE,
        Self::EQUIPMENT,
        Self::WEAPON_MODS,
        Self::WEAPONS_SPECIALIZED,
        Self::WEAPON_MODS_ENHANCED,
    ];

    pub const ITEMS_WITH_CHARACTERS: &'static [&'static str] = &[
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
    pub name: Uuid,
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

    pub category: String,
    pub attachable_categories: Vec<String>,
    pub rarity: Option<String>,
    pub cap: Option<u32>,

    pub consumable: Option<bool>,
    pub droppable: Option<bool>,
    pub deletable: Option<bool>,

    /// Name of definition that this item depends on
    /// (Requires the item to reach its capacity before it can be dropped)
    /// TODO: Handle this when doing store rewards
    pub unlock_definition: Option<Uuid>,

    #[serde(flatten)]
    pub events: ItemEvents,

    pub restrictions: Option<String>,
    pub default_namespace: String,

    #[serialize_always]
    pub secret: Option<Value>,
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
