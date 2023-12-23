use crate::{database::entity::ChallengeProgress, definitions::challenges::ChallengeDefinition};
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize)]
pub struct ChallengeCategories {
    pub categories: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct ChallengesResponse {
    pub challenges: Vec<ChallengeItem>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeItem {
    #[serde(flatten)]
    pub definition: &'static ChallengeDefinition,
    pub progress: Option<Vec<ChallengeProgress>>,
}
