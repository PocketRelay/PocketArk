use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use axum_server::tls_openssl::OpenSSLConfig;
use openssl::{
    pkey::PKey,
    rsa::Rsa,
    ssl::{SslAcceptor, SslMethod, SslVersion},
    x509::X509,
};
use tower_http::trace::TraceLayer;

pub async fn start_http() {
    let addr: SocketAddr = "0.0.0.0:443".parse().unwrap();
    let app = Router::new()
        .route("/auth", post(auth::authenticate))
        .route("/configuration", get(configuration::get_configuration))
        .layer(TraceLayer::new_for_http());
    let mut a = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();

    let crt = X509::from_der(include_bytes!("cert.der")).unwrap();
    let pkey =
        PKey::from_rsa(Rsa::private_key_from_pem(include_bytes!("key.pem")).unwrap()).unwrap();

    a.set_certificate(&crt).unwrap();
    a.set_private_key(&pkey).unwrap();
    a.set_min_proto_version(Some(SslVersion::TLS1_2)).unwrap();
    a.set_max_proto_version(Some(SslVersion::TLS1_2)).unwrap();

    let config = OpenSSLConfig::try_from(a).unwrap();

    axum_server::bind_openssl(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

mod auth {
    use axum::Json;
    use chrono::Utc;
    use log::debug;
    use uuid::Uuid;

    use crate::structs::{AuthRequest, AuthResponse, AuthUser};

    /// POST /auth
    pub async fn authenticate(Json(req): Json<AuthRequest>) -> Json<AuthResponse> {
        debug!("Authenticate: {:?}", &req);

        Json(AuthResponse {
            session_id: Uuid::new_v4(),
            user: AuthUser {
                roles: vec![
                    "GameSettings.Anonymous",
                    "Telemetry.User",
                    "User",
                    "Presence.User",
                    "CharacterStorage.User",
                    "StrikeTeams.User",
                    "Tools.User",
                    "Anonymous",
                    "Challenge.User",
                    "WorldVaultLegacy.User",
                    "Inventory.User",
                    "Auth.User",
                    "WebAPI.User",
                    "Activity.User",
                    "Bank.User",
                    "WorldVault.User",
                    "Localization.User",
                    "Leaderboards.User",
                    "Mission.User",
                    "Nemesis.User",
                    "Match.User",
                    "Friends.User",
                    "Achievements.User",
                    "ActivityFeed.User",
                    "Example.User",
                    "UserSettings.User",
                    "CharacterStorage.Anonymous",
                    "Notification.User",
                    "Store.User",
                    "Character.User",
                ]
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
                pid: 1000279946559,
                persona_id: 978651371,
                sku: req.sku,
                anonymous: false,
                name: "jacobtread".to_string(),
            },
            language: "en-us".to_string(),
            server_time: Utc::now(),
            pid: "1000279946559".to_string(),
        })
    }
}

mod configuration {
    use axum::{
        response::{IntoResponse, Response},
        Json,
    };
    use hyper::{header::CONTENT_TYPE, http::HeaderValue};

    static CONFIGURATION: &str = include_str!("definitions/min/configuration.json");

    /// GET /configuration
    pub async fn get_configuration() -> Response {
        let mut resp = CONFIGURATION.into_response();
        resp.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        resp
    }
}

mod mission {
    /// GET /mission/current
    async fn current_mission() {}

    /// GET /user/mission/:id
    async fn get_mission() {}

    /// POST /user/mission/:id/start
    async fn start_mission() {}

    /// POST /user/mission/:id/finish
    async fn finish_mission() {}
}

mod strike_teams {
    /// GET /striketeams
    async fn get() {}

    /// GET /striketeams/successRate
    async fn get_success_rate() {}
}

mod character {
    /// GET /characters
    async fn get_characters() {}

    /// GET /character/:id
    async fn get_character() {}

    /// POST /character/:id/active
    async fn set_active() {}

    /// GET /character/:id/equipment
    async fn get_character_equip() {}

    /// pUT /character/:id/equipment
    async fn update_character_equip() {}

    /// GET /character/:id/equipment/history
    async fn get_character_equip_history() {}

    /// PUT /character/:id/skillTrees
    async fn update_skill_tree() {}

    /// GET /character/classes
    async fn get_classes() {}

    /// GET /character/levelTables
    async fn get_level_tables() {}
}

mod challenge {

    /// GET /challenges
    async fn get_challenges() {}

    /// GET /challenges/user
    async fn get_user_challenges() {}
}

mod store {

    /// GET /store/catalogs
    async fn get_catalogs() {}

    /// POST /store/article
    async fn obtain_article() {}

    /// POST /store/unclaimed/claimAll
    async fn claim_unclaimed() {}

    /// GET /user/currencies
    async fn get_currencies() {}
}

mod inventory {

    /// GET /inventory
    async fn get_inventory() {}

    /// GET /inventory/definitions
    async fn get_definitions() {}

    /// PUT /inventory/seen
    async fn update_inventory_seen() {}

    /// POST /inventory/consume
    async fn consume_inventory() {}
}

mod leaderboard {
    /// GET /leaderboards
    async fn get_leaderboards() {}

    /// GET /leaderboards/:id
    async fn get_leaderboard() {}
}

mod presence {
    /// PUT /presence/session
    async fn update_session() {}
}

mod telemetry {
    /// POST /pinEvents
    async fn pin_events() {}
}

mod user_match {
    /// GET /user/match/badges
    async fn get_badges() {}

    /// GET /user/match/modifiers
    async fn get_modifiers() {}
}

mod activity {
    /// POST /activity
    async fn create_report() {}

    /// GET /activity/metadata
    async fn get_metadata() {}
}
