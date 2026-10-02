use crate::{
    definitions::currency::CurrencyType,
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct CurrencyProcessor;

impl CurrencyProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("currency", CurrencyProcessor)
    }
}

impl MissionOutcomeProcessor for CurrencyProcessor {
    fn process_mission_data<'a>(
        &self,
        _mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        // Todo: currency rewards dependant on difficulty and game wave completion
        outcome.add_reward_currency("enemytype", CurrencyType::Grind, 0);
        outcome.add_reward_currency("level_multiplier", CurrencyType::Grind, 0);
        outcome.add_reward_currency("difficulty_multiplier", CurrencyType::Grind, 0);
        outcome.add_reward_currency("enemytype_multiplier", CurrencyType::Grind, 0);
    }
}
