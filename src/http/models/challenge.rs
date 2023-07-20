use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{database::entity::ChallengeProgress, services::challenges::ChallengeDefinition};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeCategories {
    pub categories: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengesResponse {
    pub challenges: Vec<ChallengeItem>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeItem {
    #[serde(flatten)]
    pub definition: ChallengeDefinition,
    pub progress: Option<Vec<ChallengeProgress>>,
}
