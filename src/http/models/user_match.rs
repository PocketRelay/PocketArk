use serde::{Deserialize, Serialize};

use crate::services::match_data::Badge;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchBadgesResponse {
    pub total_count: usize,
    pub list: &'static [Badge],
}
