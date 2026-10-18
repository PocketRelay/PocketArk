use super::currency::CurrencyType;
use pocket_ark_core::custom_attributes::CustomAttributes;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::sync::OnceLock;

pub struct MatchModifiers {
    pub values: Vec<MatchModifier>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<MatchModifiers> = OnceLock::new();

impl MatchModifiers {
    /// Gets a static reference to the global [MatchModifiers] collection
    pub fn get() -> &'static MatchModifiers {
        STORE
            .get()
            .expect("match modifiers global must be initialized")
    }

    /// Initialize a new [MatchModifiers] instance from the provided definitions
    pub fn from_definitions(values: Vec<MatchModifier>) -> Self {
        Self { values }
    }

    /// Initialize the global instance
    pub fn initialize(value: MatchModifiers) {
        _ = STORE.set(value);
    }

    /// Finds a match modifier by `name`
    pub fn by_name(&self, name: &str) -> Option<&MatchModifier> {
        self.values
            .iter()
            // Find the specific modifier by name
            .find(|modifier| modifier.name.eq(name))
    }
}

/// Represents modifiers that can be applied to a match based
/// on certain values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifier {
    /// The name of the mission modifier this applies to
    pub name: String,
    /// The different modifiers for each value
    pub values: Vec<MatchModifierValue>,
}

impl MatchModifier {
    /// Finds an entry in the collection of modifiers wheres the
    /// modifier targets the provided `value`
    pub fn by_value(&self, value: &str) -> Option<&MatchModifierValue> {
        self.values.iter().find(|modifier| modifier.name.eq(value))
    }
}

/// Match modifier that should be applied when a specific value
/// is used `name` for the match modifier
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifierValue {
    /// Name/value that this modifier should only apply to
    /// (The value of the mission modifier)
    pub name: String,
    /// XP rewards
    pub xp_data: Option<ModifierAmount>,
    /// Currency rewards
    ///
    /// Stored as a [Vec] of tuples rather than a [serde_json::Map] because its
    /// only ever iterated and not used as a lookup map
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub currency_data: Vec<(CurrencyType, ModifierAmount)>,
    /// Additional attributes applied to the value
    pub custom_attributes: CustomAttributes,
}

/// Configures how much of something the modifier should give
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierAmount {
    /// Fixed amount of the value
    pub flat_amount: u32,
    /// Add a % multiplier of the original value
    pub additive_multiplier: f32,
}

impl ModifierAmount {
    /// Returns the amount that should be added based on
    /// the old value with the modifier
    pub fn get_amount(&self, old_value: u32) -> u32 {
        let additive_value = (old_value as f32 * self.additive_multiplier).trunc() as u32;
        self.flat_amount + additive_value
    }
}
