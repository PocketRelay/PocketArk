use crate::blaze::{
    models::user_sessions::{UpdateHardwareFlags, UpdateNetworkInfo},
    router::Blaze,
    session::SessionLink,
};

pub async fn update_network_info(session: SessionLink, Blaze(req): Blaze<UpdateNetworkInfo>) {
    let info = req.info;
    session.set_network_info(info.addr, info.qos);
}

pub async fn update_hardware_flags(session: SessionLink, Blaze(req): Blaze<UpdateHardwareFlags>) {
    session.set_hardware_flags(req.hardware_flags);
}
