use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

mod activity;
mod auth;
mod challenge;
mod character;
mod configuration;
mod inventory;
mod leaderboard;
mod mission;
mod presence;
mod store;
mod strike_teams;
mod telemetry;
mod user_match;

pub fn router() -> Router {
    Router::new()
        .route("/auth", post(auth::authenticate))
        .route("/configuration", get(configuration::get_configuration))
        .layer(TraceLayer::new_for_http())
}
