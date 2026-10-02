use crate::{
    definitions::match_modifiers::MatchModifiers,
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct ModifiersProcessor;

impl ModifiersProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("modifiers", ModifiersProcessor)
    }
}

impl MissionOutcomeProcessor for ModifiersProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        let mission_modifiers = &mission_data.mission_data.modifiers;
        let match_modifiers = MatchModifiers::get();

        mission_modifiers
            .iter()
            .filter_map(|mission_modifier| {
                // Find a matching modifier
                let match_modifier = match_modifiers.by_name(&mission_modifier.name)?;
                // Find a matching modifier value
                let modifier_value = match_modifier.by_value(&mission_modifier.value)?;

                Some((match_modifier, modifier_value))
            })
            .for_each(|(modifier, modifier_entry)| {
                // Apply xp rewards if the modifier has any
                if let Some(xp_data) = &modifier_entry.xp_data {
                    let amount = xp_data.get_amount(outcome.xp_earned);
                    outcome.add_reward_xp(&modifier.name, amount);
                }

                modifier_entry
                    .currency_data
                    .iter()
                    .for_each(|(key, modifier_data)| {
                        // Get current currency amount for additive multiplier
                        let current_amount =
                            outcome.total_currency.get(key).copied().unwrap_or_default();

                        // Get the earned amount
                        let earned_amount = modifier_data.get_amount(current_amount);
                        outcome.add_reward_currency(&modifier.name, *key, earned_amount);
                    });
            });
    }
}
