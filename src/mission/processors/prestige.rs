use crate::{
    database::dto::shared_data::SharedProgression,
    definitions::{
        i18n::{I18nDescription, I18nName},
        level_tables::LevelTables,
    },
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct PrestigeProcessor;

impl PrestigeProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("prestige", PrestigeProcessor)
    }
}

impl MissionOutcomeProcessor for PrestigeProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        let class = mission_data.class;
        let level_tables = LevelTables::get();

        let prestige_level_table = level_tables
            .by_name(&class.prestige_level_name)
            .expect("Missing prestige level table");

        let mut shared_progression = mission_data.shared_data.shared_progression.clone();

        // Insert the initial prestige data if we don't have any
        // (Needs to happen *before* append_prestige_before to ensure it shows up in the "before" state)
        if !shared_progression
            .iter()
            .any(|value| value.name.eq(&class.prestige_level_name))
        {
            shared_progression.push(SharedProgression {
                i18n_name: I18nName::raw(""),
                i18n_description: I18nDescription::raw(""),
                level: 0,
                name: class.prestige_level_name,
                xp: prestige_level_table.initial_progression(),
            });
        }

        // Insert the before change
        outcome.append_prestige_before(&shared_progression);

        // Character prestige leveling
        {
            let shared_progression = &mut shared_progression;
            let prestige_value = shared_progression
                .iter_mut()
                .find(|value| value.name.eq(&class.prestige_level_name));

            // Update the prestige value in-place
            if let Some(prestige_value) = prestige_value {
                let (new_xp, level) = prestige_level_table.compute_leveling(
                    prestige_value.xp,
                    prestige_value.level,
                    outcome.xp_earned,
                );

                prestige_value.xp = new_xp;
                prestige_value.level = level;
            }
        }

        // Insert after change
        outcome.append_prestige_after(&shared_progression);

        outcome.shared_progression = shared_progression;
    }
}
