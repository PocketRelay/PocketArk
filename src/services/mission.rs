//! Service for keeping track of creating missions and managing
//! existing missions

use std::ops::Add;

use chrono::{DateTime, Days, FixedOffset, NaiveDateTime, TimeZone, Timelike, Utc};
use sea_orm::{prelude::DateTimeUtc, ConnectionTrait, DatabaseConnection};

use crate::database::DbResult;

/// Background task that handles creating missions on the fixed
/// four hourly schedule
pub struct MissionBackgroundTask {
    /// Database access is required for missions
    db: DatabaseConnection,
}

/// Represents an hour offset for execution
type HourOffset = u32;

impl MissionBackgroundTask {
    const HOURS_IN_DAY: u32 = 24;
    const SCHEDULE_HOURLY_INTERVAL: u32 = 4;
    const TOTAL_DAILY_OFFSETS: u32 = Self::HOURS_IN_DAY / Self::SCHEDULE_HOURLY_INTERVAL;

    /// Finds the last schedule offset that was executed at if there was one
    async fn last_executed_offset(&self) -> DbResult<Option<u32>> {
        unimplemented!()
    }

    /// Finds the offset nearest to the provided `hour`
    fn offset_for_hour(&self, hour: u32) -> u32 {
        // Iterate in reverse to find the latest hour possible
        (0..Self::TOTAL_DAILY_OFFSETS)
            .rev()
            // Find a matching hour offset
            .find(|offset| {
                // Get the hour at this offset
                let offset_hour = offset * Self::SCHEDULE_HOURLY_INTERVAL;

                hour <= offset_hour
            })
            // Defaults to the first offset
            .unwrap_or_default()
    }

    /// Determines the next [DateTimeUtc] that the task should be scheduled to run at
    fn get_next_time(&self, last_offset: Option<u32>) -> DbResult<DateTimeUtc> {
        // TODO: The offical game uses EST (-5:00 UTC)
        let current_time = Utc::now();

        let Some(last_offset) = last_offset else {
            // Haven't done any offsets yet, start immediately
            return Ok(current_time);
        };

        let mut next_offset = self.offset_for_hour(current_time.hour());

        // We already processed this offset
        if next_offset == last_offset {
            // Move to next offset
            next_offset += 1;
        }

        // Completed all offsets for today
        if next_offset >= Self::TOTAL_DAILY_OFFSETS {
            let next_time = current_time
                // Update the hour to the fixed schedule offset (First offset)
                .with_hour(0)
                .expect("Invalid hour for daily offset")
                // Move it to the next day
                .add(Days::new(1));
            return Ok(next_time);
        }

        let next_time = current_time
            // Update the hour to the fixed schedule offset
            .with_hour(next_offset * Self::SCHEDULE_HOURLY_INTERVAL)
            .expect("Invalid hour for daily offset");

        Ok(next_time)
    }
}
