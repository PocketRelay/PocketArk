use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::{Uuid, uuid};

use crate::{
    database::dto::strike_team_mission::MissionAccessibility,
    definitions::{
        challenges::CurrencyReward,
        currency::CurrencyType,
        i18n::Localized,
        items::{ItemDefinition, ItemName, Items},
        strike_teams::mission::MissionDifficulty,
    },
};

pub type MissionRewardsId = Uuid;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionRewards {
    /// Unique ID for the rewards collection
    pub name: MissionRewardsId,
    /// Currency rewards from the mission
    pub currency_reward: Option<CurrencyReward>,
    /// Multiplayer items earned from the mission
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub mp_item_rewards: Vec<(ItemName, u32)>,
    /// Single player items earned from the mission
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub sp_item_rewards: Vec<(ItemName, u32)>,
    /// Definitions of the items that should be earned
    #[serde(default)]
    pub item_definitions: Option<Vec<ItemDefinition>>,
}

impl Localized for MissionRewards {
    fn localize(&mut self, i18n: &crate::definitions::i18n::I18n) {
        self.item_definitions.localize(i18n);
    }
}

impl MissionRewards {
    /// Construct new mission rewards
    pub fn new(difficulty: MissionDifficulty, accessibility: MissionAccessibility) -> Self {
        let currency_reward = CurrencyReward {
            name: CurrencyType::Mission,
            value: difficulty_currency_amount(difficulty, accessibility),
        };

        let mp_item_rewards: Vec<(ItemName, u32)> = Vec::new();
        let sp_item_rewards: Vec<(ItemName, u32)> =
            difficulty_rewards_singleplayer(difficulty, accessibility);

        let items = Items::get();
        let item_definitions = items.collect_by_name(
            mp_item_rewards
                .iter()
                .chain(sp_item_rewards.iter())
                .map(|(item, _)| item),
        );

        Self {
            name: Uuid::new_v4(),
            currency_reward: Some(currency_reward),
            mp_item_rewards,
            sp_item_rewards,
            item_definitions: Some(item_definitions),
        }
    }

    /// Construct new mission rewards
    pub fn empty() -> Self {
        let currency_reward = CurrencyReward {
            name: CurrencyType::Mission,
            value: 5,
        };

        let mp_item_rewards: Vec<(ItemName, u32)> = Vec::new();
        let sp_item_rewards: Vec<(ItemName, u32)> = Vec::new();
        let item_definitions = Vec::new();

        Self {
            name: uuid::uuid!("8d8c4dc3-d44c-4bd1-840b-9344c60f5bf6"),
            currency_reward: Some(currency_reward),
            mp_item_rewards,
            sp_item_rewards,
            item_definitions: Some(item_definitions),
        }
    }
}

/// The amount of currency rewarded based on the mission
fn difficulty_currency_amount(
    difficulty: MissionDifficulty,
    accessibility: MissionAccessibility,
) -> u32 {
    match (accessibility, difficulty) {
        // Strike team missions give 5 mission currency
        (MissionAccessibility::SinglePlayer, _) => 5,

        // Platinum multiplayer missions give 15 mission currency
        (
            MissionAccessibility::Any | MissionAccessibility::MultiPlayer,
            MissionDifficulty::Platinum,
        ) => 15,

        // Any other multiplayer mission difficulty gives you 10 currency
        (MissionAccessibility::Any | MissionAccessibility::MultiPlayer, _) => 10,
    }
}

/// Get the singleplayer rewards for the provided difficulty
/// and mode of access
fn difficulty_rewards_singleplayer(
    difficulty: MissionDifficulty,
    accessibility: MissionAccessibility,
) -> Vec<(ItemName, u32)> {
    match accessibility {
        MissionAccessibility::Any | MissionAccessibility::MultiPlayer => {
            multiplayer_difficulty_rewards_singleplayer(difficulty)
        }
        MissionAccessibility::SinglePlayer => {
            singleplayer_difficulty_rewards_singleplayer(difficulty)
        }
    }
}

/// Get the multiplayer difficulty rewards to give the singleplayer mode
fn multiplayer_difficulty_rewards_singleplayer(
    difficulty: MissionDifficulty,
) -> Vec<(ItemName, u32)> {
    match difficulty {
        MissionDifficulty::Bronze => {
            vec![
                // "Bronze Item Loot Box"
                (uuid!("14d5e5ba-dbb5-4336-ad07-607eb39409bb"), 1),
                // "Research Data Loot Box"
                (uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1),
            ]
        }
        MissionDifficulty::Silver => {
            vec![
                // "Silver Item Loot Box"
                (uuid!("a7d46d7a-1f42-4eac-b106-c2fb96aa3e7a"), 1),
                // "Research Data Loot Box"
                (uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1),
            ]
        }
        MissionDifficulty::Gold | MissionDifficulty::Platinum => {
            vec![
                // "Gold Item Loot Box"
                (uuid!("58383d3f-d74d-4518-b27e-988f56ade54c"), 1),
                // "Research Data Loot Box"
                (uuid!("71c483fd-371f-4dd4-b9a1-11f189322972"), 1),
            ]
        }
    }
}

/// Get the singleplayer difficulty rewards to give the singleplayer mode
fn singleplayer_difficulty_rewards_singleplayer(
    difficulty: MissionDifficulty,
) -> Vec<(ItemName, u32)> {
    match difficulty {
        MissionDifficulty::Bronze => {
            vec![
                // "Bronze Credit Loot Box"
                (uuid!("e300500e-885e-4ee5-bbdc-f706b30b362a"), 1),
                // "Bronze Material Loot Box"
                (uuid!("1440d464-0245-49f9-8533-4930b9283d78"), 1),
            ]
        }
        MissionDifficulty::Silver => {
            vec![
                // "Silver Credit Loot Box"
                (uuid!("e4556800-5eef-d487-182f-5044f0f2d534"), 1),
                // "Silver Material Loot Box"
                (uuid!("004f85aa-f7ac-4262-8109-e7e7d6d94bd5"), 1),
            ]
        }
        MissionDifficulty::Gold => {
            vec![
                // "Gold Credit Loot Box"
                (uuid!("9860be4d-b3b2-445f-aa7d-1728fc163ddb"), 1),
                // "Silver Material Loot Box"
                (uuid!("61d3f563-ad29-4f97-9c80-71c72549a5fe"), 1),
            ]
        }

        // Platinum mission should *never* be single player (Strike team) missions
        // it is not possible normally
        MissionDifficulty::Platinum => Vec::new(),
    }
}
