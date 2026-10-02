use crate::{
    http::models::mission::{CompleteMissionData, MissionPlayerData},
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{
            badges::BadgeProcessor, initial_score::InitialScoreProcessor,
            initial_xp::InitialXpProcessor, modifiers::ModifiersProcessor,
        },
    },
};

use std::sync::LazyLock;

pub mod badges;
pub mod initial_score;
pub mod initial_xp;
pub mod modifiers;

pub struct ProcessMissionData<'a> {
    pub player_data: &'a MissionPlayerData,
    pub mission_data: &'a CompleteMissionData,
}

pub trait MissionOutcomeProcessor: Send + Sync {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    );
}

pub struct ProcessorPipelineEntry {
    pub name: Box<str>,
    pub processor: Box<dyn MissionOutcomeProcessor>,
}

impl ProcessorPipelineEntry {
    pub fn new<T: MissionOutcomeProcessor + 'static>(name: impl AsRef<str>, processor: T) -> Self {
        Self {
            name: Box::from(name.as_ref()),
            processor: Box::new(processor),
        }
    }
}

pub static MISSION_PROCESSOR_PIPELINE: LazyLock<[ProcessorPipelineEntry; 4]> =
    LazyLock::new(|| {
        [
            InitialScoreProcessor::entry(),
            BadgeProcessor::entry(),
            InitialXpProcessor::entry(),
            ModifiersProcessor::entry(),
        ]
    });
