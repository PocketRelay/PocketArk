use uuid::Uuid;

use crate::{
    database::dto::{character::CharacterDto, shared_data::SharedDataDto},
    definitions::classes::Class,
    http::models::mission::{CompleteMissionData, MissionPlayerData},
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{
            badges::BadgeProcessor, challenges::ChallengesProcessor,
            character_leveling::CharacterLevelingProcessor, currency::CurrencyProcessor,
            initial_score::InitialScoreProcessor, initial_xp::InitialXpProcessor,
            modifiers::ModifiersProcessor, prestige::PrestigeProcessor,
        },
    },
};

use std::sync::LazyLock;

pub mod badges;
pub mod challenges;
pub mod character_leveling;
pub mod currency;
pub mod initial_score;
pub mod initial_xp;
pub mod modifiers;
pub mod prestige;

pub struct ProcessMissionData<'a> {
    pub player_data: &'a MissionPlayerData,
    pub mission_data: &'a CompleteMissionData,
    pub class: &'a Class,
    pub character: &'a CharacterDto,
    pub active_character_id: Uuid,
    pub shared_data: &'a SharedDataDto,
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

pub static MISSION_PROCESSOR_PIPELINE: LazyLock<[ProcessorPipelineEntry; 8]> =
    LazyLock::new(|| {
        [
            InitialScoreProcessor::entry(),
            BadgeProcessor::entry(),
            InitialXpProcessor::entry(),
            ModifiersProcessor::entry(),
            CharacterLevelingProcessor::entry(),
            PrestigeProcessor::entry(),
            CurrencyProcessor::entry(),
            ChallengesProcessor::entry(),
        ]
    });
