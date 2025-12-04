use std::{any::type_name, boxed::Box};

use sea_orm::{
    TryGetableFromJson,
    sea_query::{ArrayType, ColumnType, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub mod challenge_progress;
pub mod characters;
pub mod currency;
pub mod inventory_items;
pub mod seen_articles;
pub mod shared_data;
pub mod strike_team_mission;
pub mod strike_team_mission_progress;
pub mod strike_teams;
pub mod users;

pub type Character = characters::Model;
pub type ChallengeProgress = challenge_progress::Model;
pub type Currency = currency::Model;
pub type SharedData = shared_data::Model;
pub type InventoryItem = inventory_items::Model;
pub type User = users::Model;
pub type StrikeTeam = strike_teams::Model;
pub type StrikeTeamMission = strike_team_mission::Model;
pub type StrikeTeamMissionProgress = strike_team_mission_progress::Model;

/// Wrapper around a generic [serde_json::Map]
pub type SeaGenericMap = SeaJson<serde_json::Map<String, serde_json::Value>>;

/// Wrapper around JSON serializable types that allows them to be used
/// as value types for SeaORM
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SeaJson<T>(pub T);

impl<T> SeaJson<T> {
    #[allow(unused)]
    fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for SeaJson<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for SeaJson<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> TryGetableFromJson for SeaJson<T> where T: DeserializeOwned {}

impl<T> From<SeaJson<T>> for sea_orm::Value
where
    T: Serialize,
{
    fn from(value: SeaJson<T>) -> Self {
        sea_orm::Value::Json(serde_json::to_value(&value).ok().map(Box::new))
    }
}

impl<T> ValueType for SeaJson<T>
where
    T: DeserializeOwned,
{
    fn try_from(v: sea_orm::Value) -> Result<Self, ValueTypeErr> {
        match v {
            sea_orm::Value::Json(Some(json)) => {
                serde_json::from_value(*json).map_err(|_| ValueTypeErr)
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        type_name::<T>().to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::Json
    }
}
