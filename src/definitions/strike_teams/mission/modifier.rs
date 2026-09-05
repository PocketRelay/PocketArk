use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    definitions::strike_teams::mission::{MissionDifficulty, level::MissionLevel, tag::MissionTag},
    utils::ImStr,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionModifier {
    /// The name of the modifier ("difficulty", "enemyType", "level", etc)
    pub name: ImStr,
    /// The value of the modifier
    pub value: ImStr,
}

impl MissionModifier {
    const STATIC_DIFFICULTY: &str = "difficulty";
    const STATIC_ENEMY_TYPE: &str = "enemyType";
    const STATIC_LEVEL: &str = "level";

    fn new(name: impl Into<ImStr>, value: impl Into<ImStr>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn static_modifiers(
        difficulty: MissionDifficulty,
        enemy_tag: &MissionTag,
        level: MissionLevel,
    ) -> Vec<MissionModifier> {
        vec![
            MissionModifier::new(Self::STATIC_DIFFICULTY, difficulty.to_string()),
            MissionModifier::new(Self::STATIC_ENEMY_TYPE, enemy_tag.name.clone()),
            MissionModifier::new(Self::STATIC_LEVEL, level.to_string()),
        ]
    }

    pub fn dynamic_modifiers<R>(_rng: &mut R) -> anyhow::Result<Vec<MissionModifier>>
    where
        R: Rng,
    {
        // TODO: Randomly select mission modifiers
        Ok(vec![])
    }
}
