use crate::blaze::{
    models::user_sessions::{NetworkInfo, UpdateHardwareFlags, UpdateNetworkRequest},
    router::Blaze,
    session::SessionLink,
};

pub async fn update_network_info(
    session: SessionLink,
    Blaze(UpdateNetworkRequest { info }): Blaze<UpdateNetworkRequest>,
) {
    let NetworkInfo {
        address,
        ping_site_latency,
        qos,
    } = info;

    // TODO: Additional network handling checks for Qos types
    let ping_site_latency: Vec<u32> = if let Some(ping_site_latency) = ping_site_latency {
        ping_site_latency.values().copied().collect()
    } else {
        Vec::new()
    };

    session
        .data
        .set_network_info(address, qos, ping_site_latency);
}

pub async fn update_hardware_flags(session: SessionLink, Blaze(req): Blaze<UpdateHardwareFlags>) {
    session.data.set_hardware_flags(req.hardware_flags);
}
