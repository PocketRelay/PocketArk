use super::shared::CustomAttributes;
use crate::database::entity::currency::CurrencyType;
use anyhow::Context;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::sync::OnceLock;

/// Modifier definitions (6)
pub const MATCH_MODIFIER_DEFINITIONS: &str = include_str!("../resources/data/matchModifiers.json");

pub struct MatchModifiers {
    pub values: Vec<MatchModifier>,
}

/// Static storage for the definitions once its loaded
/// (Allows the definitions to be passed with static lifetimes)
static STORE: OnceLock<MatchModifiers> = OnceLock::new();

impl MatchModifiers {
    /// Gets a static reference to the global [MatchModifiers] collection
    pub fn get() -> &'static MatchModifiers {
        STORE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> anyhow::Result<Self> {
        let values: Vec<MatchModifier> = serde_json::from_str(MATCH_MODIFIER_DEFINITIONS)
            .context("Failed to load match modifier definitions")?;

        debug!("Loaded {} match modifier definition(s)", values.len(),);

        Ok(Self { values })
    }

    /// Finds a match modifier by `name`
    #[allow(unused)]
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
    #[allow(unused)]
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
    #[allow(unused)]
    pub fn get_amount(&self, old_value: u32) -> u32 {
        let adative_value = (old_value as f32 * self.additive_multiplier).trunc() as u32;
        self.flat_amount + adative_value
    }
}

#[cfg(test)]
mod test {
    use super::MatchModifiers;

    /// Tests ensuring loading succeeds
    #[test]
    fn ensure_load_succeed() {
        _ = MatchModifiers::load().unwrap();
    }
}
