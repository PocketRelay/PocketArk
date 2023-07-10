use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

mod activity;
mod auth;
mod challenge;
mod character;
mod client;
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
        .route("/ark/client/auth", get(client::authenticate))
        .route("/ark/client/details", get(client::details))
        .route("/ark/client/upgrade", get(client::upgrade))
        .route("/auth", post(auth::authenticate))
        .route("/configuration", get(configuration::get_configuration))
        .layer(TraceLayer::new_for_http())
}
