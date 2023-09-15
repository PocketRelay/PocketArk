use crate::blaze::{
    models::user_sessions::{UpdateHardwareFlags, UpdateNetworkInfo},
    router::Blaze,
    session::SessionLink,
};

pub async fn update_network_info(session: SessionLink, Blaze(req): Blaze<UpdateNetworkInfo>) {
    tokio::spawn(async move {
        let info = req.info;
        session.set_network_info(info.addr, info.qos).await;
    });
}

pub async fn update_hardware_flags(session: SessionLink, Blaze(req): Blaze<UpdateHardwareFlags>) {
    tokio::spawn(async move {
        session.set_hardware_flags(req.hardware_flags).await;
    });
}
