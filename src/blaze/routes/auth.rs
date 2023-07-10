use futures::SinkExt;

use crate::blaze::{
    components,
    models::auth::{AuthNotify, AuthRequest, AuthResponse},
    pk::packet::Packet,
    session::Session,
};

pub async fn auth(state: &mut Session, req: AuthRequest) -> AuthResponse {
    state
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::UPDATE_AUTH,
            AuthNotify,
        ))
        .await;

    AuthResponse
}
