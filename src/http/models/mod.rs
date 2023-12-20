use axum::response::{IntoResponse, Response};
use hyper::{header::CONTENT_TYPE, http::HeaderValue};
use serde::Serialize;
use std::fmt::Debug;

pub mod auth;
pub mod challenge;
pub mod character;
pub mod client;
pub mod errors;
pub mod inventory;
pub mod leaderboard;
pub mod mission;
pub mod qos;
pub mod store;
pub mod strike_teams;
pub mod telemetry;
pub mod user_match;

pub use errors::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWithCount<V>
where
    V: Debug + Sized + Serialize + 'static,
{
    pub total_count: usize,
    pub list: &'static [V],
}

impl<V> ListWithCount<V>
where
    V: Debug + Sized + Serialize + 'static,
{
    pub fn new(list: &'static [V]) -> Self {
        Self {
            total_count: list.len(),
            list,
        }
    }
}

/// Raw pre encoded JSON string response
pub struct RawJson(pub &'static str);

impl IntoResponse for RawJson {
    fn into_response(self) -> Response {
        let mut res = self.0.into_response();
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        res
    }
}
