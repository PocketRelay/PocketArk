use tdf::TdfMap;

use crate::blaze::router::Blaze;
use crate::blaze::session::SessionLink;
use crate::blaze::{models::util::*, router::SessionAuth};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Handles responding to pre-auth requests which is the first request
/// that clients will send. The response to this contains information
/// about the server the client is connecting to.
pub async fn pre_auth(session: SessionLink) -> Blaze<PreAuthResponse> {
    Blaze(PreAuthResponse)
}

/// Handles post authentication requests. This provides information about other
/// servers that are used by Mass Effect such as the Telemetry and Ticker servers.
pub async fn post_auth(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
) -> Blaze<PostAuthResponse> {
    // Subscribe to the session with itself
    session
        .data
        .add_subscriber(user.id, Arc::downgrade(&session));

    Blaze(PostAuthResponse { user_id: user.id })
}

/// Handles the client requesting to fetch a configuration from the server.
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

/// Handles ping update requests. These are sent by the client at the interval
/// specified in the pre-auth response. The server replies to this messages with
/// the current server unix timestamp in seconds.
pub async fn ping(session: SessionLink) -> Blaze<PingResponse> {
    session.data.set_alive();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    Blaze(PingResponse { time })
}
