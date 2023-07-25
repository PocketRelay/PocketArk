use crate::{database::entity::InventoryItem, services::items::ItemDefinition};
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;
use uuid::Uuid;

/// Paramas for requesting inventory
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct InventoryRequestQuery {
    /// Whether to include definitions in the response
    pub include_definitions: bool,
    /// Optional namespace to filter by
    pub namespace: Option<String>,
}

/// Response containing all the inventory items and their definitions
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct InventoryResponse {
    /// List of inventory items
    pub items: Vec<InventoryItem>,
    /// Definitions for items (only present when asked for in query)
    pub definitions: Option<Vec<&'static ItemDefinition>>,
}

/// Response containing all the item definitions
#[derive(Debug, Serialize)]
pub struct ItemDefinitionsResponse {
    pub total_count: usize,
    pub list: &'static [ItemDefinition],
}

/// Request updating inventory item seen states
#[derive(Debug, Deserialize)]
pub struct InventorySeenRequest {
    /// The list of item IDs to mark as seen
    pub list: Vec<Uuid>,
}

/// Item consume request body
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeRequest {
    /// List of items to consume
    pub items: Vec<ConsumeTarget>,
    /// The namespace to search within
    pub namespace: String,
}

/// Target item that should be consumed
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeTarget {
    /// ID of the item to consume
    pub item_id: Uuid,
    // pub target_id: String, *unused*
}
