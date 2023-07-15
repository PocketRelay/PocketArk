use axum::Json;
use chrono::Utc;
use log::debug;
use uuid::Uuid;

use crate::{
    http::models::auth::{AuthRequest, AuthResponse, AuthUser},
    services::tokens::Tokens,
};

/// POST /auth
pub async fn authenticate(Json(req): Json<AuthRequest>) -> Json<AuthResponse> {
    debug!("Authenticate: {:?}", &req);

    let token = Tokens::service_claim(1);

    Json(AuthResponse {
        session_id: token,
        user: AuthUser {
            roles: &[
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
            ],
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
