use crate::http::models::strike_teams::StrikeTeamMissionWithState;

/// Tutorial mission definition
const TUTORIAL_MISSION_DEFINITION: &str =
    include_str!("../../resources/data/tutorial_mission.json");

pub fn get_tutorial_mission() -> StrikeTeamMissionWithState {
    serde_json::from_str(TUTORIAL_MISSION_DEFINITION).expect("failed to load tutorial mission data")
}
