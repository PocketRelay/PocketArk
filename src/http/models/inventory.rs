use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct InventorySeenList {
    pub list: Vec<Uuid>,
}
