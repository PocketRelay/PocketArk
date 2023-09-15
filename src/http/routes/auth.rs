use axum::{Extension, Json};
use chrono::Utc;
use hyper::StatusCode;
use log::debug;
use sea_orm::DatabaseConnection;

use crate::{
    database::entity::User,
    http::models::{
        auth::{AuthRequest, AuthResponse, AuthUser},
        HttpError, HttpResult,
    },
    services::tokens::Tokens,
    state::App,
};

/// POST /auth
pub async fn authenticate(
    Extension(db): Extension<DatabaseConnection>,
    Json(req): Json<AuthRequest>,
) -> HttpResult<AuthResponse> {
    debug!("Authenticate: {:?}", &req);

    let user = User::get_user(&db, req.persona_id)
        .await?
        .ok_or(HttpError::new("Invalid user", StatusCode::BAD_REQUEST))?;

    let token = Tokens::service_claim(req.persona_id);

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
