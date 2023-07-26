use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    services::{activity::ActivityResult, character::Xp},
    utils::models::LocaleNameWithDesc,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveMissionResponse {
    pub team: Team,
    pub mission_successful: bool,
    pub traits_acquired: Vec<Value>,
    pub activity_response: ActivityResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub icon: TeamIcon,
    pub level: u32,
    pub xp: Xp,
    pub positive_traits: Vec<TeamTrait>,
    pub negative_traits: Vec<TeamTrait>,
    pub out_of_date: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamIcon {
    pub name: String,
    pub image: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTrait {
    pub name: String,
    pub tag: String,
    pub effectiveness: u32,

    #[serde(flatten)]
    pub locale: LocaleNameWithDesc,
}
