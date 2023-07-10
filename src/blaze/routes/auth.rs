use futures::SinkExt;

use crate::blaze::{
    components,
    models::auth::{AuthNotify, AuthRequest, AuthResponse},
    pk::packet::Packet,
    session::Session,
};

pub async fn auth(session: &mut Session, req: AuthRequest) -> AuthResponse {
    let _ = session
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::UPDATE_AUTH,
            AuthNotify,
        ))
        .await;

    AuthResponse
}

pub async fn list_entitlements_2(session: &mut Session) {}
