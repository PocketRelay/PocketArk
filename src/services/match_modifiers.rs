use crate::database::entity::currency::CurrencyType;
use anyhow::Context;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, sync::OnceLock};

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

    pub fn by_name(
        &self,
        name: &str,
        value: &str,
    ) -> Option<(&MatchModifier, &MatchModifierEntry)> {
        let modifier = self
            .values
            .iter()
            // Find the specific modifier by name
            .find(|modifier| modifier.name.eq(name))?;

        // Find the modifier value by the desired value
        let value = modifier.values.iter().find(|entry| entry.name.eq(value))?;

        Some((modifier, value))
    }
}

/// Represents modifiers that can be applied to a match based
/// on certain values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifier {
    pub name: String,
    pub values: Vec<MatchModifierEntry>,
}

/// Represents a level of a match modifier
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifierEntry {
    pub name: String,
    pub xp_data: Option<ModifierData>,
    pub currency_data: HashMap<CurrencyType, ModifierData>,
    pub custom_attributes: serde_json::Map<String, serde_json::Value>,
}

/// Configuration for how the modifier should be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierData {
    pub flat_amount: u32,
    pub additive_multiplier: f32,
}

impl ModifierData {
    /// Returns the amount that should be added based on
    /// the old value with the modifier
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
