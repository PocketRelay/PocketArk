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
        .nest(
            "/ark/client",
            Router::new()
                .route("/auth", post(client::authenticate))
                .route("/details", get(client::details))
                .route("/upgrade", get(client::upgrade)),
        )
        .route("/auth", post(auth::authenticate))
        .route("/configuration", get(configuration::get_configuration))
        .route("/mission/current", get(mission::current_missions))
        .nest(
            "/striketeams",
            Router::new()
                .route("/", get(strike_teams::get))
                .route("/successRate", get(strike_teams::get_success_rate)),
        )
        .route("/characters", get(character::get_characters))
        .nest(
            "/character",
            Router::new()
                .nest(
                    "/:id",
                    Router::new()
                        .route("/", get(character::get_character))
                        .route("/active", post(character::set_active))
                        .route(
                            "/customization",
                            post(character::update_character_customization),
                        )
                        .nest(
                            "/equipment",
                            Router::new()
                                .route(
                                    "/",
                                    get(character::get_character_equip)
                                        .put(character::update_character_equip),
                                )
                                .route("/history", get(character::get_character_equip_history)),
                        )
                        .route("/skillTrees", put(character::update_skill_tree)),
                )
                .route("/unlocked", post(character::character_unlocked))
                .route("/classes", get(character::get_classes))
                .route("/levelTables", get(character::get_level_tables)),
        )
        .nest(
            "/store",
            Router::new()
                .route("/catalogs", get(store::get_catalogs))
                .route("/article", post(store::obtain_article))
                .route("/article/seen", post(store::update_seen_articles))
                .route("/unclaimed/claimAll", post(store::claim_unclaimed)),
        )
        .nest(
            "/user",
            Router::new()
                .route("/currencies", get(store::get_currencies))
                .nest(
                    "/match",
                    Router::new()
                        .route("/badges", get(user_match::get_badges))
                        .route("/modifiers", get(user_match::get_modifiers)),
                )
                .nest(
                    "/mission",
                    Router::new().nest(
                        "/:id",
                        Router::new()
                            .route("/", get(mission::get_mission))
                            .route("/start", post(mission::start_mission))
                            .route("/finish", post(mission::finish_mission)),
                    ),
                ),
        )
        .nest(
            "/challenges",
            Router::new()
                .route("/", get(challenge::get_challenges))
                .route("/user", get(challenge::get_user_challenges))
                .route("/categories", get(challenge::get_challenge_categories)),
        )
        .nest(
            "/activity",
            Router::new()
                .route("/", post(activity::create_report))
                .route("/metadata", get(activity::get_metadata)),
        )
        .nest(
            "/inventory",
            Router::new()
                .route("/", get(inventory::get_inventory))
                .route("/definitions", get(inventory::get_definitions))
                .route("/seen", put(inventory::update_inventory_seen))
                .route("/consume", post(inventory::consume_inventory)),
        )
        .route("//em/v3/*path", any(ok))
        .route("/presence/session", put(presence::update_session))
        .route("/pinEvents", post(telemetry::pin_events))
        .layer(TraceLayer::new_for_http())
}

async fn ok() -> Response {
    StatusCode::OK.into_response()
}
