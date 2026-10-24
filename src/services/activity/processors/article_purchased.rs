use crate::services::{
    activity::{ActivityResult, processors::ActivityProcessor},
    items::{AcquireItem, acquire_item},
};
use async_trait::async_trait;
use pocket_ark_database::{DbTransaction, dto::users::UserDto};
use pocket_ark_definitions::{
    classes::Classes,
    items::Items,
    level_tables::LevelTables,
    store_catalogs::{StoreArticleName, StoreCatalogs},
};
use serde::Deserialize;
use thiserror::Error;

/// Handles events for purchasing items from the store
pub struct ArticlePurchasedActivityProcessor;

#[derive(Debug, Deserialize)]
#[serde(rename = "camelCase")]
pub struct ArticlePurchasedAttributes {
    article_name: StoreArticleName,
    count: u32,
}

/// Errors that can occur while processing an
/// article purchase
#[derive(Debug, Error)]
pub enum ArticlePurchaseError {
    /// Couldn't find the article requested
    #[error("Unknown article")]
    UnknownArticle,
    /// Server definition error, article associated item was
    /// not present in the item definitions
    #[error("Unknown article item")]
    UnknownArticleItem,
}

#[async_trait]
impl ActivityProcessor for ArticlePurchasedActivityProcessor {
    type Event = ArticlePurchasedAttributes;

    async fn process_event(
        &self,
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: Self::Event,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        let catalogs = StoreCatalogs::get();
        let item_definitions = Items::get();
        let classes = Classes::get();
        let level_tables = LevelTables::get();

        // Find the article we are looking for
        let article = catalogs
            .catalog
            .get_article(&event.article_name)
            // Article doesn't exist anymore
            .ok_or(ArticlePurchaseError::UnknownArticle)?;

        // Find the item given by the article
        let item_definition = item_definitions
            .by_name(&article.item_name)
            .ok_or(ArticlePurchaseError::UnknownArticleItem)?;

        // Give the user the article item
        let item = acquire_item(
            classes,
            level_tables,
            db,
            user.id,
            AcquireItem {
                definition: item_definition,
                stack_size: event.count,
            },
        )
        .await?;

        // Append the change to the result
        result.add_item(item, event.count, item_definition);

        Ok(())
    }
}
