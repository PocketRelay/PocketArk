//! Service in charge of deailing with items opening packs

use rand::rngs::StdRng;
use sea_orm::DatabaseTransaction;

use crate::database::{
    entity::{InventoryItem, User},
    DbResult,
};

pub struct PackBuilder {}

struct GuarenteeItem {
    def: String,
    stack_size: u32,
}

impl GuarenteeItem {
    async fn grant_item(
        &self,
        rng: &mut StdRng,
        user: &User,
        tx: &DatabaseTransaction,
    ) -> DbResult<InventoryItem> {
        let mut item =
            InventoryItem::create_or_append(tx, user, self.def.to_string(), self.stack_size)
                .await?;
        item.stack_size = self.stack_size;

        Ok(item)
    }
}

struct RngFromCategory {
    category: String,
    amount: u32,
    stack_size: u32,
}

impl PackBuilder {
    pub fn new() -> Self {
        Self {}
    }
}
