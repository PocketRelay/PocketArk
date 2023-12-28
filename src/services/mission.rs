//! Service for keeping track of creating missions and managing
//! existing missions

use std::{ops::Add, time::Duration};

use anyhow::Context;
use chrono::{Datelike, Days, TimeZone, Timelike, Utc};
use log::error;
use rand::{rngs::StdRng, SeedableRng};
use sea_orm::{prelude::DateTimeUtc, DatabaseConnection};
use tokio::time::sleep;

use crate::{
    database::entity::StrikeTeamMission,
    definitions::strike_teams::{random_mission, MissionDifficulty, StrikeTeamMissionData},
};

/// Background task that handles creating missions on the fixed
/// four hourly schedule
pub struct MissionBackgroundTask {
    /// Database access is required for missions
    db: DatabaseConnection,
}

/// Represents an hour offset for execution
type HourOffset = u32;

impl MissionBackgroundTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Starts the task in a background tokio task
    pub fn start(self) {
        tokio::spawn(async move {
            self.run();
        });
    }

    const HOURS_IN_DAY: u32 = 24;
    const SCHEDULE_HOURLY_INTERVAL: u32 = 4;
    const TOTAL_DAILY_OFFSETS: u32 = Self::HOURS_IN_DAY / Self::SCHEDULE_HOURLY_INTERVAL;

    /// Finds the date time of the last created mission
    async fn last_mission_time(&self) -> anyhow::Result<Option<DateTimeUtc>> {
        let start_seconds: u64 = match StrikeTeamMission::newest_mission(&self.db).await {
            Ok(Some(value)) => value,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let date_time = Utc
            .timestamp_opt(start_seconds as i64, 0)
            .single()
            .context("Failed to determine last mission date")?;

        Ok(Some(date_time))
    }

    /// Finds the offset nearest to the provided `hour`
    fn offset_for_hour(hour: u32) -> Option<HourOffset> {
        // Iterate in reverse to find the latest hour possible
        (1..=Self::TOTAL_DAILY_OFFSETS)
            .rev()
            // Find a matching hour offset
            .find(|offset| {
                // Get the hour at this offset
                let offset_hour = (offset * Self::SCHEDULE_HOURLY_INTERVAL) - 1;
                hour <= offset_hour
            })
    }

    /// Gets all the offsets between and including the two hour values.
    /// Used to get missed offsets
    fn inclusive_offsets(first: HourOffset, second: HourOffset) -> Vec<HourOffset> {
        (first..=second)
            // Filter only offset values
            .filter(|value| (*value) % Self::SCHEDULE_HOURLY_INTERVAL == 0)
            // Collect the results
            .collect()
    }

    async fn run(&self) {
        let mut failures = 0;

        loop {
            if let Err(err) = self.process().await {
                error!("Error while processing mission background task: {}", err);

                failures += 1;

                // Stop trying if we already failed 10 times without success
                if failures == 10 {
                    break;
                }

                // Debounce waiting every failure to prevent quickly looping the same failure
                sleep(Duration::from_secs(failures * 5)).await;
            } else {
                // Reset failures on successful attempt
                failures = 0;
            }
        }
    }

    async fn process(&self) -> anyhow::Result<()> {
        let current_time = Utc::now();

        let last_date_time = self
            .last_mission_time()
            .await
            .context("Failed to get last mission creation time")?;

        // Whether the last execution happened this same day
        let last_is_today = last_date_time.is_some_and(|last| last.day() == current_time.day());

        let last_offset = last_date_time
            .as_ref()
            // Last offset should only be computed if it is from today
            .filter(|_| last_is_today)
            // Find the offset for the last date
            .and_then(|last| Self::offset_for_hour(last.hour()));

        // Get the next offset to execute at
        let next_offset = match Self::get_next_offset(&current_time, last_offset) {
            Some(value) => value,
            // No more offsets available for today (Sleep till next day)
            None => {
                // Restart processing a day from now at the first hour
                let next_date = current_time.with_hour(0).unwrap().add(Days::new(1));
                Self::sleep_until(next_date).await?;
                return Ok(());
            }
        };

        // Determine how long to sleep for till the next offset
        let next_date = current_time
            // Update the hour to the fixed schedule offset
            .with_hour((next_offset * Self::SCHEDULE_HOURLY_INTERVAL) - 1)
            .expect("Invalid hour for daily offset");

        Self::sleep_until(next_date).await?;

        // Determine the offsets that should be computed (Gets all the missed offsets since last)
        let offsets = Self::inclusive_offsets(last_offset.unwrap_or_default(), next_offset);

        for offset in offsets {
            self.create_mission_offset(offset).await?;
        }

        Ok(())
    }

    /// Creates new missions for the provided `offset`
    async fn create_mission_offset(&self, offset: HourOffset) -> anyhow::Result<()> {
        const AM_4: HourOffset = 1;
        const AM_8: HourOffset = 2;
        const AM_12: HourOffset = 3;
        const PM_4: HourOffset = 4;
        const PM_8: HourOffset = 5;
        const PM_12: HourOffset = 6;

        let mut rng = StdRng::from_entropy();

        // Mission data to create
        let mut mission_data: Vec<StrikeTeamMissionData> = Vec::new();

        // Bronze standard issued at 12am and 12pm
        if offset == AM_12 || offset == PM_12 {
            // Bronze Standard
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Bronze, false)?);
        }

        // Silver standard issued at 4am and 4pm
        if offset == AM_4 || offset == PM_4 {
            // Silver Standard
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Silver, false)?);
        }

        // Gold standard issued at 8am and 8pm
        if offset == AM_8 || offset == PM_8 {
            // Gold Standard
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Gold, false)?);
        }

        // Bronze apex issued at 12am
        if offset == AM_12 {
            // Bronze Apex
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Bronze, true)?);
        }

        // Gold apex issued at 4pm
        if offset == PM_4 {
            // Gold Apex
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Gold, true)?);
        }

        // Silver and platinum apex issued at 8pm
        if offset == PM_8 {
            // Silver Apex
            // Platinum Apex
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Silver, true)?);
            mission_data.push(random_mission(&mut rng, MissionDifficulty::Platinum, true)?);
        }

        StrikeTeamMission::create_many(&self.db, mission_data)
            .await
            .context("Failed to create strike team missions")?;

        Ok(())
    }

    /// Sleeps until the provided date time is reached
    async fn sleep_until(date: DateTimeUtc) -> anyhow::Result<()> {
        let now = Utc::now();

        // Already passed the date
        if date.lt(&now) {
            return Ok(());
        }

        // Get the duration to sleep
        let duration = date
            .signed_duration_since(now)
            .to_std()
            .context("Sleep timing was out of range for task")?;

        sleep(duration).await;
        Ok(())
    }

    /// Returns the next offset for the current time, if there is another
    /// offset available for the date
    fn get_next_offset(
        current_time: &DateTimeUtc,
        last_offset: Option<HourOffset>,
    ) -> Option<HourOffset> {
        let mut next_offset = Self::offset_for_hour(current_time.hour())?;

        // We already processed this offset
        if last_offset.is_some_and(|last| last == next_offset) {
            // Move to next offset
            next_offset += 1;
        }

        if next_offset == Self::TOTAL_DAILY_OFFSETS {
            None
        } else {
            Some(next_offset)
        }
    }
}
