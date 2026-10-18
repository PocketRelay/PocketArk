use anyhow::Context;
use pocket_ark_database::{
    DbTransaction,
    dto::{
        strike_teams::{CreateStrikeTeamDto, StrikeTeamDto},
        users::UserDto,
    },
    repositories::strike_teams::StrikeTeamsRepository,
};
use pocket_ark_definitions::strike_teams::{StrikeTeams, random_strike_team};
use rand::{SeedableRng, rngs::StdRng};
use std::ops::DerefMut;

/// Creates a new strike team for the provided user
pub async fn create_user_strike_team(
    strike_teams: &StrikeTeams,
    db: &mut DbTransaction<'_>,
    user: &UserDto,
) -> anyhow::Result<StrikeTeamDto> {
    // Generate random strike team data
    let mut rng = StdRng::from_entropy();
    let strike_team_data =
        random_strike_team(strike_teams, &mut rng).context("Failed to create strike team")?;

    // Create the strike team
    let team = StrikeTeamsRepository::create(
        db.deref_mut(),
        CreateStrikeTeamDto {
            user_id: user.id,
            name: strike_team_data.name,
            icon: strike_team_data.icon,
            level: strike_team_data.level,
            xp: strike_team_data.xp,
            positive_traits: vec![strike_team_data.positive_trait],
            negative_traits: Vec::new(),
        },
    )
    .await?;
    Ok(team)
}
