use futures::{sink::Flush, SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use log::{error, info};
use tokio_util::codec::Framed;
use uuid::Uuid;

use crate::http::middleware::upgrade::UpgradedTarget;

use super::{
    models::PreAuthResponse,
    packet::{Packet, PacketCodec},
    router::Router,
};

pub struct Session {
    pub host_target: UpgradedTarget,
    pub io: Framed<Upgraded, PacketCodec>,
    pub id: Uuid,
}

pub fn router() -> Router<Session> {
    let mut router = Router::new();

    router.route((9, 7), pre_auth);

    router
}

pub async fn handle_session(mut session: Session) {
    let router = router();

    while let Some(Ok(packet)) = session.io.next().await {
        let s = match router.handle(&mut session, packet) {
            Ok(value) => value,
            Err(err) => {
                error!("{:?}", err);
                continue;
            }
        };
        let s = s.await;
        session.io.send(s).await;
        (session.io.flush() as Flush<'_, Framed<Upgraded, PacketCodec>, Packet>).await;
    }
}

async fn pre_auth(state: &mut Session) -> PreAuthResponse {
    info!("pre auth request");
    PreAuthResponse {
        target: state.host_target.clone(),
    }
}
