use crate::mission::{
    outcome::MissionOutcomeDataBuilder,
    processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
};

pub struct InitialScoreProcessor;

impl InitialScoreProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("initial score", InitialScoreProcessor)
    }
}

impl MissionOutcomeProcessor for InitialScoreProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        outcome.score = mission_data
            .player_data
            .activity_report
            .activity_total_score();
    }
}
