use axum::{
    error_handling::HandleErrorLayer,
    response::{IntoResponse, Response},
    routing::{any, get, post, put},
    Router,
};
use hyper::StatusCode;
use tower::ServiceBuilder;
use tower_http::{
    decompression::RequestDecompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

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
mod qos;
mod store;
mod strike_teams;
mod telemetry;
mod user_match;

pub fn router() -> Router {
    Router::new()
        .nest(
            "/api/server",
            Router::new()
                .route("/", get(client::details))
                .route("/login", post(client::login))
                .route("/create", post(client::create))
                .route("/upgrade", get(client::upgrade)),
        )
        .route("/auth", post(auth::authenticate))
        .route("/configuration", get(configuration::get_configuration))
        .nest(
            "/mission",
            Router::new()
                .route("/current", get(mission::current_missions))
                .route("/seen", put(mission::update_seen)),
        )
        .nest(
            "/striketeams",
            Router::new()
                .route("/", get(strike_teams::get))
                .route("/successRate", get(strike_teams::get_success_rate))
                .route("/missionConfig", get(strike_teams::get_mission_config))
                .route("/specializations", get(strike_teams::get_specializations))
                .route("/equipment", get(strike_teams::get_equipment))
                .route("/{id}/mission/resolve", post(strike_teams::resolve_mission))
                .route("/{id}/mission/{id}", get(strike_teams::get_mission))
                .route("/{id}/retire", post(strike_teams::retire))
                .route(
                    "/{id}/equipment/{name}",
                    post(strike_teams::purchase_equipment),
                )
                .route("/purchase", post(strike_teams::purchase)),
        )
        .route("/characters", get(character::get_characters))
        .nest(
            "/character",
            Router::new()
                .nest(
                    "/{id}",
                    Router::new()
                        .route("/", get(character::get_character))
                        .route("/active", post(character::set_active))
                        .route(
                            "/customization",
                            put(character::update_character_customization),
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
                .route("/equipment/shared", put(character::update_shared_equip))
                .route("/unlocked", post(character::character_unlocked))
                .route("/classes", get(character::get_classes))
                .route("/levelTables", get(character::get_level_tables)),
        )
        .nest(
            "/store",
            Router::new()
                .route("/catalogs", get(store::get_catalogs))
                .route("/article", post(store::obtain_article))
                .route("/article/seen", put(store::update_seen_articles))
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
                        "/{id}",
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
        .route("//em/v3/{*path}", any(ok))
        .route("/presence/session", put(presence::update_session))
        .route("/pinEvents", post(telemetry::pin_events))
        .nest(
            "/leaderboards",
            Router::new()
                .route("/", get(leaderboard::get_leaderboards))
                .route("/{id}", get(leaderboard::get_leaderboard)),
        )
        .route("/wv/playthrough/0", put(activity::update_playthrough))
        .nest(
            "/qos",
            Router::new()
                .route("/qos", get(qos::qos_query))
                .route("/firewall", get(qos::qos_firewall))
                .route("/firetype", get(qos::qos_firetype)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_error| async move {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled server error")
                }))
                .layer(RequestDecompressionLayer::new()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new())
                .on_response(DefaultOnResponse::new()),
        )

    // .layer(CompressionLayer::new())
}

async fn ok() -> Response {
    StatusCode::OK.into_response()
}
