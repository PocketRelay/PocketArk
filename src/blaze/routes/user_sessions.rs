use std::sync::Arc;

use crate::{
    blaze::{
        models::{
            errors::ServerResult,
            user_sessions::{
                LookupRequest, LookupResponse, NetworkInfo, UpdateHardwareFlags,
                UpdateNetworkRequest, UserSessionsError,
            },
        },
        router::{Blaze, Extension},
        session::SessionLink,
    },
    services::sessions::Sessions,
};

pub async fn update_network_info(
    session: SessionLink,
    Blaze(UpdateNetworkRequest { info }): Blaze<UpdateNetworkRequest>,
) {
    let NetworkInfo {
        mut address,
        ping_site_latency,
        qos,
    } = info;

    // TODO: Additional network handling checks for Qos types
    let ping_site_latency: Vec<u32> = if let Some(ping_site_latency) = ping_site_latency {
        ping_site_latency.values().copied().collect()
    } else {
        Vec::new()
    };

    match &mut address {
        crate::blaze::models::user_sessions::NetworkAddress::AddressPair(ip_pair_address) => {
            ip_pair_address.external = ip_pair_address.internal.clone();
        }
        crate::blaze::models::user_sessions::NetworkAddress::Unset => todo!(),
        crate::blaze::models::user_sessions::NetworkAddress::Default => todo!(),
    }

    session
        .data
        .set_network_info(address, qos, ping_site_latency);
}

pub async fn update_hardware_flags(session: SessionLink, Blaze(req): Blaze<UpdateHardwareFlags>) {
    session.data.set_hardware_flags(req.hardware_flags);
}

/// Attempts to lookup another authenticated session details
pub async fn lookup_user(
    Blaze(req): Blaze<LookupRequest>,
    Extension(sessions): Extension<Arc<Sessions>>,
) -> ServerResult<Blaze<LookupResponse>> {
    // Lookup the session
    let session = sessions
        .lookup_session(req.player_id)
        .ok_or(UserSessionsError::UserNotFound)?;

    // Get the lookup response from the session
    let response = session
        .data
        .get_lookup_response()
        .ok_or(UserSessionsError::UserNotFound)?;

    Ok(Blaze(response))
}
