use futures::SinkExt;

use crate::blaze::{
    components,
    models::{
        auth::{AuthNotify, AuthRequest, AuthResponse},
        user_sessions::UserAdded,
    },
    pk::packet::Packet,
    session::Session,
};

pub async fn auth(session: &mut Session, _req: AuthRequest) -> AuthResponse {
    let _ = session
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::UPDATE_AUTH,
            AuthNotify,
        ))
        .await;
    let _ = session
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::USER_ADDED,
            UserAdded {
                player_id: 1,
                name: "Jacobtread".to_string(),
                game_id: session.data.game,
                net_data: session.data.net.clone(),
            },
        ))
        .await;

    AuthResponse
}

pub async fn list_entitlements_2(session: &mut Session) {}
