//! Shared commonly used type definitions

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Collection of custom attributes
#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct CustomAttributes(
    #[serde_as(as = "serde_with::Map<_, _>")] Vec<(String, serde_json::Value)>,
);

impl CustomAttributes {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0
            .iter()
            // Find matching key
            .find(|(k, v)| k.eq(key))
            // Only return value
            .map(|(_, v)| v)
    }
}
