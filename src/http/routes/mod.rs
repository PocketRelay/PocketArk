use axum::{
    response::{IntoResponse, Response},
    routing::{any, get, post, put},
    Router,
};
use hyper::StatusCode;
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
        .route("/user/match/badges", get(user_match::get_badges))
        .route("/user/match/modifiers", get(user_match::get_modifiers))
        .route("/mission/current", get(mission::current_mission))
        .route("/user/mission/:id", get(mission::get_mission))
        .route("/user/mission/:id/start", post(mission::start_mission))
        .route("/user/mission/:id/finish", post(mission::finish_mission))
        .route("/striketeams", get(strike_teams::get))
        .route(
            "/striketeams/successRate",
            get(strike_teams::get_success_rate),
        )
        .route("/characters", get(character::get_characters))
        .route("/character/:id", get(character::get_character))
        .route("/character/:id/active", post(character::set_active))
        .route(
            "/character/:id/equipment",
            get(character::get_character_equip).put(character::update_character_equip),
        )
        .route(
            "/character/:id/equipment/history",
            get(character::get_character_equip_history),
        )
        .route(
            "/character/:id/skillTrees",
            put(character::update_skill_tree),
        )
        .route("/character/classes", get(character::get_classes))
        .route("/character/levelTables", get(character::get_level_tables))
        .route("/store/catalogs", get(store::get_catalogs))
        .route("/store/article", post(store::obtain_article))
        .route("/store/unclaimed/claimAll", post(store::claim_unclaimed))
        .route("/user/currencies", get(store::get_currencies))
        .route("/challenges", get(challenge::get_challenges))
        .route("/challenges/user", get(challenge::get_user_challenges))
        .route(
            "/challenges/categories",
            get(challenge::get_challenge_categories),
        )
        .route("/activity", post(activity::create_report))
        .route("/activity/metadata", get(activity::get_metadata))
        .route("//em/v3/*path", any(ok))
        .route("/presence/session", put(presence::update_session))
        .route("/pinEvents", post(telemetry::pin_events))
        .route("/inventory", get(inventory::get_inventory))
        .route("/inventory/definitions", get(inventory::get_definitions))
        .route("/inventory/seen", put(inventory::update_inventory_seen))
        .route("/inventory/consume", post(inventory::consume_inventory))
        .layer(TraceLayer::new_for_http())
}

async fn ok() -> Response {
    StatusCode::OK.into_response()
}
