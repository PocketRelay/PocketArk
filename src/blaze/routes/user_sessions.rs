use crate::blaze::{
    models::user_sessions::{UpdateHardwareFlags, UpdateNetworkInfo},
    router::Blaze,
    session::{HardwareFlagsMessage, NetworkInfoMessage, SessionLink},
};

pub async fn update_network_info(session: SessionLink, Blaze(req): Blaze<UpdateNetworkInfo>) {
    let info = req.info;
    let _ = session
        .send(NetworkInfoMessage {
            addr: info.addr,
            qos: info.qos,
        })
        .await;
}

pub async fn update_hardware_flags(session: SessionLink, Blaze(req): Blaze<UpdateHardwareFlags>) {
    let _ = session
        .send(HardwareFlagsMessage { flags: req.flags })
        .await;
}
