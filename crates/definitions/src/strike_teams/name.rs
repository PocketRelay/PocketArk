use anyhow::Context;
use rand::{Rng, seq::SliceRandom};

/// Collection of names that strike teams are randomly named from
///
/// Sourced from "NATO phonetic alphabet"
static STRIKE_TEAM_NAMES: &[&str] = &[
    "Yankee", "Delta", "India", "Echo", "Zulu", "Charlie", "Whiskey", "Lima", "Bravo", "Sierra",
    "November", "X-Ray", "Golf", "Alpha", "Romeo", "Kilo", "Tango", "Quebec", "Foxtrot", "Papa",
    "Mike", "Oscar", "Juliet", "Uniform", "Victor", "Hotel",
];

/// Type alias for the name of a strike team
pub type StrikeTeamName = String;

/// Chooses a random strike team name from [STRIKE_TEAM_NAMES]
pub fn random_team_name<R>(rng: &mut R) -> anyhow::Result<StrikeTeamName>
where
    R: Rng,
{
    STRIKE_TEAM_NAMES
        .choose(rng)
        .context("Failed to choose name")
        .map(|value| value.to_string())
}
