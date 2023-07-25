use serde::Serialize;

use crate::services::match_data::{Badge, MatchModifier};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchBadgesResponse {
    pub total_count: usize,
    pub list: &'static [Badge],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchModifiersResponse {
    pub total_count: usize,
    pub list: &'static [MatchModifier],
}
