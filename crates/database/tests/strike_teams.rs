use pocket_ark_database::{
    dto::strike_teams::CreateStrikeTeamDto, repositories::strike_teams::StrikeTeamsRepository,
};
use pocket_ark_definitions::{level_tables::ProgressionXp, strike_teams::icon::StrikeTeamIcon};

use crate::helpers::{mock_user, test_database};

pub mod helpers;

/// Tests that initially the get_by_user query should succeed and produce no
/// items for a fresh user
#[tokio::test]
async fn tests_strike_teams_can_be_created() {
    let db = test_database().await;
    let user = mock_user(&db, "test@test.com", "test").await;

    let _value = StrikeTeamsRepository::create(
        &db,
        CreateStrikeTeamDto {
            user_id: user.id,
            name: "test".to_string(),
            icon: StrikeTeamIcon {
                name: "test".to_string().into_boxed_str(),
                image: "test".to_string().into_boxed_str(),
            },
            level: 0,
            xp: ProgressionXp::default(),
            positive_traits: Vec::new(),
            negative_traits: Vec::new(),
        },
    )
    .await
    .unwrap();
}
