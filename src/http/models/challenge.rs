use crate::{
    database::entity::{
        ChallengeProgress,
        challenge_progress::{ChallengeProgressCounterWithDefinition, ChallengeState},
    },
    definitions::challenges::{ChallengeDefinition, ChallengeDefinitionBase},
};
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize)]
pub struct ChallengeCategories {
    pub categories: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct UserChallengesResponse {
    pub challenges: Vec<UserChallengeItem>,
}

#[derive(Debug, Serialize)]
pub struct AllChallengesResponse {
    pub challenges: Vec<ChallengeAllItem>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeAllItem {
    #[serde(flatten)]
    pub definition: &'static ChallengeDefinition,
    pub progress: Option<Vec<ChallengeProgress>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserChallengeItem {
    #[serde(flatten)]
    pub definition: &'static ChallengeDefinitionBase,
    pub counters: Vec<ChallengeProgressCounterWithDefinition>,
    pub state: ChallengeState,
    pub times_completed: u32,
    pub last_completed: Option<String>,
    pub first_completed: Option<String>,
    pub last_changed: String,
    pub rewarded: bool,
}
