use crate::{
    definitions::badges::{BadgeLevelName, Badges},
    http::models::mission::PlayerInfoBadge,
    mission::{
        outcome::MissionOutcomeDataBuilder,
        processors::{MissionOutcomeProcessor, ProcessMissionData, ProcessorPipelineEntry},
    },
};

pub struct BadgeProcessor;

impl BadgeProcessor {
    pub fn entry() -> ProcessorPipelineEntry {
        ProcessorPipelineEntry::new("badges", BadgeProcessor)
    }
}

impl MissionOutcomeProcessor for BadgeProcessor {
    fn process_mission_data<'a>(
        &self,
        mission_data: &'a ProcessMissionData<'a>,
        outcome: &mut MissionOutcomeDataBuilder,
    ) {
        let badges = Badges::get();
        let activities = &mission_data.player_data.activity_report.activities;

        activities
            .iter()
            // Find matching badges for the activity
            .filter_map(|activity| {
                // Find a badge matching the activity
                let (badge, progress, levels) = badges.by_activity(activity)?;
                // Only continue if they have a level achieved
                let highest_level = *levels.last()?;

                Some((badge, progress, levels, highest_level))
            })
            .for_each(|(badge, progress, levels, highest_level)| {
                // Total accumulated XP and currency from achieved levels
                let mut total_xp: u32 = 0;
                let mut total_currency: u32 = 0;

                // Names of the levels that have been earned
                let mut level_names: Vec<BadgeLevelName> = Vec::with_capacity(levels.len());

                for level in levels {
                    total_xp += level.xp_reward;
                    total_currency += level.currency_reward;
                    level_names.push(level.name.clone());
                }

                // The reward source is the badge name
                let reward_name = badge.name.to_string();

                // Append the rewards
                outcome.add_reward_xp(&reward_name, total_xp);
                outcome.add_reward_currency(&reward_name, badge.currency, total_currency);
                outcome.badges.push(PlayerInfoBadge {
                    count: progress,
                    level_name: highest_level.name.clone(),
                    rewarded_levels: level_names,
                    name: badge.name,
                });
            });
    }
}
