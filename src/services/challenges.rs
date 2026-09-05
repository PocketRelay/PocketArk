use chrono::{DateTime, Utc};

use crate::{
    database::dto::challenge_progress::{
        ChallengeProgressCounter, ChallengeProgressDto, ChallengeState, CounterUpdateType,
    },
    definitions::challenges::{ChallengeCounter, ChallengeDefinition},
    services::game::data::ChallengeProgressChange,
};

pub struct AppliedChallengeProgressUpdate {
    pub last_changed: DateTime<Utc>,
    pub times_completed: u32,
    pub counters: Vec<ChallengeProgressCounter>,
    //
    pub first_completed: Option<DateTime<Utc>>,
    pub last_completed: Option<DateTime<Utc>>,
    pub state: ChallengeState,
}

pub fn apply_challenge_progress_change(
    challenge: &ChallengeProgressDto,
    change: &ChallengeProgressChange,
) -> (
    AppliedChallengeProgressUpdate,
    CounterUpdateType,
    ChallengeProgressCounter,
) {
    let now = Utc::now();

    // Take all the counters from the original list
    let mut counters = challenge.counters.clone();

    let update_type: CounterUpdateType;

    // Find the counter if it already exists
    let counter = if let Some(existing) = counters
        .iter_mut()
        .find(|counter| counter.name == change.counter.name)
    {
        update_type = CounterUpdateType::Changed;
        existing
    } else {
        // Create a new counter
        update_type = CounterUpdateType::Created;

        counters.push(ChallengeProgressCounter::new(change.counter.name.clone()));

        counters
            .last_mut()
            .expect("Counter was just inserted but is missing")
    };

    let prev_completion_times = counter.times_completed;

    // Add and update the progression
    add_counter_progress(counter, change.progress);
    process_counter_state(counter, change.definition, change.counter);
    counter.last_changed = now;

    // Take a copy of the counter for re-use
    let counter = counter.clone();

    // First completion
    let first_completion = prev_completion_times == 0 && counter.times_completed > 0;
    // Challenge counter was completed
    let completed = prev_completion_times != counter.times_completed;

    let first_completed = if first_completion {
        Some(now)
    } else {
        challenge.first_completed
    };

    let (last_completed, state) = if completed {
        (Some(now), ChallengeState::Completed)
    } else {
        (challenge.last_completed, challenge.state)
    };

    (
        AppliedChallengeProgressUpdate {
            last_changed: now,
            times_completed: counter.times_completed,
            counters,
            first_completed,
            last_completed,
            state,
        },
        update_type,
        counter,
    )
}

/// Add progress to the provided counter
fn add_counter_progress(counter: &mut ChallengeProgressCounter, progress: u32) {
    counter.total_count = counter.total_count.saturating_add(progress);
    counter.current_count = counter.current_count.saturating_add(progress);
}

/// Processes the counter state ensuring that the times completed
/// and current count are adjusted
fn process_counter_state(
    counter: &mut ChallengeProgressCounter,
    definition: &ChallengeDefinition,
    counter_definition: &ChallengeCounter,
) {
    if definition.base.can_repeat {
        // Handle repeating the task multiple times
        while counter.current_count >= counter_definition.target_count {
            // Remove the completed amount
            counter.current_count -= counter_definition.target_count;
            // Increase the times completed
            counter.times_completed += 1;
        }
    } else if counter.current_count > counter_definition.target_count {
        counter.current_count = counter_definition.target_count;
        counter.times_completed = 1;
    }
}
