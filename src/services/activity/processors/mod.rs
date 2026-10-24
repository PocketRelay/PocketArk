use std::{collections::HashMap, sync::LazyLock};

use async_trait::async_trait;
use pocket_ark_database::{DbTransaction, dto::users::UserDto};
use pocket_ark_definitions::activity::{ActivityEvent, ActivityName};
use serde::de::DeserializeOwned;

use crate::services::activity::{
    ActivityResult,
    processors::{
        article_purchased::ArticlePurchasedActivityProcessor,
        item_consumed::ItemConsumedActivityProcessor,
    },
};

mod article_purchased;
mod item_consumed;

/// Strongly typed activity processor
#[async_trait]
trait ActivityProcessor: Send + Sync + 'static {
    type Event: DeserializeOwned;

    async fn process_event(
        &self,
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: Self::Event,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()>;
}

/// [ActivityProcessor] with the event type erased the inner implementation
/// of this handles parsing the activity event to the inner implementation
/// expected attributes type
#[async_trait]
trait ErasedActivityProcessor: Send + Sync + 'static {
    async fn process_event(
        &self,
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()>;
}

#[async_trait]
impl<Event: DeserializeOwned, P: ActivityProcessor<Event = Event>> ErasedActivityProcessor for P
where
    Event: Send + Sync + 'static,
{
    async fn process_event(
        &self,
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        event: ActivityEvent,
        result: &mut ActivityResult,
    ) -> anyhow::Result<()> {
        let event: Event = event.parse_attributes()?;
        self.process_event(db, user, event, result).await
    }
}

/// Helper function to create erased activity processors
fn processor<E: DeserializeOwned + Send + Sync + 'static, P: ActivityProcessor<Event = E>>(
    processor: P,
) -> Box<dyn ErasedActivityProcessor> {
    Box::new(processor)
}

/// Global defined available processors
static ACTIVITY_PROCESSORS: LazyLock<HashMap<ActivityName, Box<dyn ErasedActivityProcessor>>> =
    LazyLock::new(|| {
        [
            (
                ActivityName::ItemConsumed,
                processor(ItemConsumedActivityProcessor),
            ),
            (
                ActivityName::ArticlePurchased,
                processor(ArticlePurchasedActivityProcessor),
            ),
        ]
        .into()
    });

/// Process an activity event using the specific processor
#[tracing::instrument(
    name = "process_event",
    skip_all,
    fields(
        user_id = %user.id,
        event = ?event
    )
)]
pub async fn process_activity_event(
    db: &mut DbTransaction<'_>,
    user: &UserDto,
    event: ActivityEvent,
    result: &mut ActivityResult,
) -> anyhow::Result<()> {
    let processor = match ACTIVITY_PROCESSORS.get(&event.name) {
        Some(value) => value,
        None => {
            tracing::debug!("no activity processor for event");
            return Ok(());
        }
    };

    processor.process_event(db, user, event, result).await?;
    Ok(())
}
