use crate::mission::{
    outcome::MissionOutcomeDataBuilder,
    processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
};

pub struct InitialXpProcessor;

impl InitialXpProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("initial xp", InitialXpProcessor)
    }
}

impl MissionOutcomeProcessor for InitialXpProcessor {
    fn process_mission_data<'a>(
        &self,
        _mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        // Base reward xp is the score earned
        outcome.add_reward_xp("base", outcome.score);

        // TODO: "other_badge_rewards"
        outcome.add_reward_xp("other_badge_rewards", 0);
    }
}
