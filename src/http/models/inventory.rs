use super::HttpError;
use crate::{
    database::entity::{inventory_items::ItemId, InventoryItem},
    definitions::items::{InventoryNamespace, ItemDefinition},
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InventoryError {
    /// User doesn't own the item they tried to consume
    #[error("The user does not own the item.")]
    NotOwned,

    /// User doesn't own enough of the item
    #[error("Not enough of owned item")]
    NotEnough,

    /// Tried to consume a non-consumable item
    #[error("Item not consumable")]
    NotConsumable,

    /// Internal server error because item definition was missing
    #[error("Item missing definition")]
    MissingDefinition,
}

impl HttpError for InventoryError {
    fn status(&self) -> StatusCode {
        match self {
            InventoryError::NotOwned => StatusCode::NOT_FOUND,
            InventoryError::NotConsumable => StatusCode::BAD_REQUEST,
            InventoryError::NotEnough => StatusCode::CONFLICT,
            InventoryError::MissingDefinition => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Paramas for requesting inventory
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InventoryRequestQuery {
    /// Whether to include definitions in the response
    pub include_definitions: bool,
    /// Optional namespace to filter by
    pub namespace: Option<InventoryNamespace>,
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
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct InventorySeenRequest {
    /// The list of item IDs to mark as seen
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub list: Vec<ItemId>,
}

/// Item consume request body
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct ConsumeRequest {
    /// List of items to consume
    pub items: Vec<ConsumeTarget>,
    /// The namespace to search within
    pub namespace: String,
}

/// Target item that should be consumed
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeTarget {
    /// ID of the item to consume
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub item_id: ItemId,
    // pub target_id: String, *unused*
}
