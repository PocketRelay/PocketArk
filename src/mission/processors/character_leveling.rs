use crate::{
    definitions::level_tables::LevelTables,
    mission::{
        outcome::{MissionOutcomeDataBuilder, MissionOutcomeLevelingChange},
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct CharacterLevelingProcessor;

impl CharacterLevelingProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("character leveling", CharacterLevelingProcessor)
    }
}

impl MissionOutcomeProcessor for CharacterLevelingProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        let level_tables = LevelTables::get();

        // Character leveling
        let level_table = level_tables
            .by_name(&mission_data.class.level_name)
            .expect("Missing class level table");

        let character = mission_data.character;
        let previous_xp = character.xp;
        let previous_level = character.level;

        let (xp, level) =
            level_table.compute_leveling(character.xp, character.level, outcome.xp_earned);

        if xp != previous_xp || level > previous_level {
            outcome.leveling = Some(MissionOutcomeLevelingChange { level, xp });
        }

        // TOD: Character leveling up needs to add 3 skill points per level
    }
}
