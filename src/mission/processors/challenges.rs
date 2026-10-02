use crate::{
    definitions::challenges::Challenges,
    mission::{
        outcome::{ChallengeProgressChange, MissionOutcomeDataBuilder},
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct ChallengesProcessor;

impl ChallengesProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("challenges", ChallengesProcessor)
    }
}

impl MissionOutcomeProcessor for ChallengesProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        let activities = &mission_data.player_data.activity_report.activities;

        let challenge_definitions = Challenges::get();

        activities
            .iter()
            // Find activities with associated challenges
            .filter_map(|activity| {
                let (definition, counter, descriptor) =
                    challenge_definitions.get_by_activity(activity)?;
                // Only include activities with current progress
                let progress = activity.attribute_u32(&descriptor.progress_key).ok()?;

                Some((definition, counter, progress))
            })
            .for_each(|(definition, counter, progress)| {
                // Store the challenge changes
                outcome.add_challenge_progress(ChallengeProgressChange {
                    definition,
                    counter,
                    progress,
                })
            });
    }
}
