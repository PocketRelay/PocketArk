use crate::{
    http::{
        middleware::user::Auth,
        models::{
            auth::{AuthRequest, AuthResponse, AuthUser},
            HttpResult,
        },
    },
    services::tokens::Tokens,
};
use axum::{Extension, Json};
use chrono::Utc;
use log::debug;

/// POST /auth
pub async fn authenticate(
    Auth(user): Auth,
    Json(req): Json<AuthRequest>,
) -> HttpResult<AuthResponse> {
    debug!("Authenticate: {:?}", &req);

    let token = Tokens::service_claim(user.id);

    Ok(Json(AuthResponse {
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
            pid: user.id,
            persona_id: user.id,
            sku: req.sku,
            anonymous: false,
            name: user.username,
        },
        language: "en-us".to_string(),
        server_time: Utc::now(),
        pid: user.id.to_string(),
    }))
}
