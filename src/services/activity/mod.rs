//! The game and server publish different "Activities" which are used for tracking
//! things like progression, challenges, and how much rewards should be given
//!
//! The [ActivityService] should process these activities and update stored information
//! and rewards accordingly

use crate::services::activity::{processors::process_activity_event, result::ActivityResult};
use pocket_ark_database::{
    DbTransaction, dto::users::UserDto, repositories::currency::CurrencyRepository,
};
use pocket_ark_definitions::activity::ActivityEvent;
use std::ops::DerefMut;

pub mod processors;
pub mod result;

pub struct ActivityService;

impl ActivityService {
    pub async fn process_events(
        db: &mut DbTransaction<'_>,
        user: &UserDto,
        events: Vec<ActivityEvent>,
    ) -> anyhow::Result<ActivityResult> {
        let mut result = ActivityResult::default();

        for event in events {
            process_activity_event(db, user, event, &mut result).await?;
        }

        // Update the current user currencies
        result.currencies = CurrencyRepository::get_by_user(db.deref_mut(), user.id).await?;
        Ok(result)
    }
}
