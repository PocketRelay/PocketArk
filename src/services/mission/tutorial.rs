use pocket_ark_database::dto::strike_team_mission::{StrikeTeamMissionDto, UserMissionState};

use crate::{http::models::strike_teams::StrikeTeamMissionWithState, resources::Resources};

/// Path to the default inventory JSON file resource
const TUTORIAL_MISSION_RESOURCE: &str = "tutorialMission.json";

/// Load the default inventory definition
pub async fn load_tutorial_mission() -> anyhow::Result<StrikeTeamMissionDto> {
    let resource = Resources::get_resource_bytes(TUTORIAL_MISSION_RESOURCE).await?;
    let parsed = serde_json::from_slice(&resource)?;
    Ok(parsed)
}

/// Get the tutorial mission along with the mission state (Hard coded as completed)
pub async fn get_tutorial_mission_state() -> anyhow::Result<StrikeTeamMissionWithState> {
    let mission = load_tutorial_mission().await?;
    Ok(StrikeTeamMissionWithState {
        mission,
        completed: true,
        seen: true,
        user_mission_state: UserMissionState::Completed,
    })
}

#[cfg(test)]
mod test {
    use crate::services::mission::tutorial::load_tutorial_mission;

    #[tokio::test]
    async fn test_load_tutorial_mission() {
        let _result = load_tutorial_mission()
            .await
            .expect("should be able to load tutorial mission");
    }
}
