use futures::SinkExt;

use crate::blaze::{
    components,
    models::user_sessions::{UpdateHardwareFlags, UpdateNetworkInfo, UserUpdated},
    pk::packet::Packet,
    session::Session,
};

pub async fn update_network_info(session: &mut Session, req: UpdateNetworkInfo) {
    session.data.net.addr = req.addr;
    session.data.net.qos = req.qos;

    let _ = session
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::USER_UPDATED,
            UserUpdated {
                player_id: 1,
                game_id: session.data.game,
                net_data: session.data.net.clone(),
            },
        ))
        .await;
}

pub async fn update_hardware_flags(session: &mut Session, req: UpdateHardwareFlags) {
    session.data.net.hwfg = req.flags;

    let _ = session
        .io
        .send(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::USER_UPDATED,
            UserUpdated {
                player_id: 1,
                game_id: session.data.game,
                net_data: session.data.net.clone(),
            },
        ))
        .await;
}
