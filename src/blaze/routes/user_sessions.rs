use crate::blaze::{
    models::user_sessions::{UpdateHardwareFlags, UpdateNetworkInfo},
    session::{HardwareFlagsMessage, NetworkInfoMessage, SessionLink},
};

pub async fn update_network_info(session: &mut SessionLink, req: UpdateNetworkInfo) {
    let info = req.info;
    let _ = session
        .send(NetworkInfoMessage {
            addr: info.addr,
            qos: info.qos,
        })
        .await;
}

pub async fn update_hardware_flags(session: &mut SessionLink, req: UpdateHardwareFlags) {
    let _ = session
        .send(HardwareFlagsMessage { flags: req.flags })
        .await;
}
