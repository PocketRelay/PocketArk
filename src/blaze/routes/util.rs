use tdf::TdfMap;

use crate::blaze::models::util::*;
use crate::blaze::router::Blaze;
use crate::blaze::session::SessionLink;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn pre_auth(session: SessionLink) -> Blaze<PreAuthResponse> {
    Blaze(PreAuthResponse)
}

pub async fn post_auth() -> Blaze<PostAuthResponse> {
    Blaze(PostAuthResponse)
}

pub async fn fetch_client_config(
    Blaze(req): Blaze<ClientConfigRequest>,
) -> Blaze<ClientConfigResponse> {
    let config: TdfMap<&'static str, &'static str> = match req.id.as_str() {
        "IdentityParams" => [
            ("display", "console2/welcome"),
            ("redirect_uri", "http://127.0.0.1/success"),
        ]
        .into_iter()
        .collect(),
        _ => TdfMap::new(),
    };

    Blaze(ClientConfigResponse { config })
}

pub async fn ping() -> Blaze<PingResponse> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    Blaze(PingResponse { time })
}
