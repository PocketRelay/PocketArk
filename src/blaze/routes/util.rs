use tdf::TdfMap;

use crate::blaze::models::util::*;
use crate::blaze::session::{GetHostTarget, SessionLink};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn pre_auth(session: &mut SessionLink) -> PreAuthResponse {
    let target = session
        .send(GetHostTarget)
        .await
        .expect("Session closed before handling");

    PreAuthResponse { target }
}

pub async fn post_auth(_session: &mut SessionLink) -> PostAuthResponse {
    PostAuthResponse
}

pub async fn fetch_client_config(
    _session: &mut SessionLink,
    req: ClientConfigRequest,
) -> ClientConfigResponse {
    let config: TdfMap<&'static str, &'static str> = match req.id.as_str() {
        "IdentityParams" => [
            ("display", "console2/welcome"),
            ("redirect_uri", "http://127.0.0.1/success"),
        ]
        .into_iter()
        .collect(),
        _ => TdfMap::new(),
    };

    ClientConfigResponse { config }
}

pub async fn ping(_session: &mut SessionLink) -> PingResponse {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    PingResponse { time }
}
