use std::sync::Arc;

use crate::{
    http::{
        middleware::{user::Auth, JsonDump},
        models::{
            auth::{AuthRequest, AuthResponse, AuthUser},
            HttpResult,
        },
    },
    services::sessions::Sessions,
};
use axum::{Extension, Json};
use chrono::Utc;
use log::debug;

/// POST /auth
pub async fn authenticate(
    Auth(user): Auth,
    Extension(sessions): Extension<Arc<Sessions>>,
    JsonDump(req): JsonDump<AuthRequest>,
) -> HttpResult<AuthResponse> {
    debug!("Authenticate: {:?}", &req);

    let token = sessions.create_token(user.id);

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
