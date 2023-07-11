use std::future::ready;

use crate::blaze::pk::packet::PacketDebug;
use crate::blaze::pk::reader::TdfReader;
use futures::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use log::{debug, error};
use tokio_util::codec::Framed;
use uuid::Uuid;

use crate::http::middleware::upgrade::UpgradedTarget;

use crate::blaze::pk::{
    packet::{Packet, PacketCodec},
    router::HandleError,
};

use super::models::user_sessions::NetData;
use super::routes::router;

pub struct Session {
    pub host_target: UpgradedTarget,
    pub io: Framed<Upgraded, PacketCodec>,
    pub id: Uuid,
    pub data: SessionData,
}

#[derive(Debug, Default)]
pub struct SessionData {
    pub net: NetData,
    pub game: Option<u32>,
}

impl Session {
    pub fn new(io: Framed<Upgraded, PacketCodec>, target: UpgradedTarget) -> Self {
        Self {
            id: Uuid::new_v4(),
            io,
            host_target: target,
            data: SessionData::default(),
        }
    }

    pub async fn handle(mut self) {
        // TODO: static router impl
        let router = router();

        while let Some(Ok(packet)) = self.io.next().await {
            debug_log_packet("READ", &packet);

            let res_fut = match router.handle(&mut self, packet) {
                Ok(value) => value,

                Err(HandleError::MissingHandler(packet)) => {
                    error!("No handler for packet: {:?}", &packet);
                    let packet = packet.respond_empty();
                    Box::pin(ready(packet))
                }
                Err(HandleError::Decoding(err)) => {
                    error!("Error while decoding packet: {}", err);
                    continue;
                }
            };
            let packet = res_fut.await;
            debug_log_packet("WRITE", &packet);
            self.io.send(packet).await.unwrap();
        }
    }
}

pub fn debug_log_packet(dir: &str, packet: &Packet) {
    let out = PacketDebug {
        packet,
        minified: false,
    };
    debug!("{}: {:?}", dir, out);
}
