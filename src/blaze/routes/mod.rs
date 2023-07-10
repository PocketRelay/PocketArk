use std::{
    alloc::System,
    future::ready,
    time::{SystemTime, UNIX_EPOCH},
};

use blaze_pk::{
    codec::{Decodable, Encodable},
    reader::TdfReader,
};
use futures::{io::Empty, sink::Flush, SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use log::{debug, error, info};
use tokio_util::codec::Framed;
use uuid::Uuid;

use crate::http::middleware::upgrade::UpgradedTarget;

use super::{
    models::{
        AuthNotify, AuthRequest, AuthResponse, ClientConfigReq, ClientConfigRes, PingResponse,
        PostAuthRes, PreAuthResponse,
    },
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
    router.route((9, 8), post_auth);
    router.route((9, 2), ping);
    router.route((1, 10), auth);
    router.route((9, 1), cfg);
    router.route((0, 0), keep_alive);

    router
}

pub async fn handle_session(mut session: Session) {
    let router = router();

    while let Some(Ok(packet)) = session.io.next().await {
        debug!("READ: {:?}", packet);

        let mut reader = TdfReader::new(&packet.body);
        let mut out = String::new();

        out.push_str("{\n");

        // Stringify the content or append error instead
        if let Err(err) = reader.stringify(&mut out) {
            // return Ok("Failed to decode".to_string());
        }

        if out.len() == 2 {
            // Remove new line if nothing else was appended
            out.pop();
        }

        out.push('}');

        debug!("Content:{}\n\n", out);

        let s = match router.handle(&mut session, packet) {
            Ok(value) => value,
            Err(err) => {
                error!("{:?}", err);
                match err {
                    super::router::HandleError::MissingHandler(packet) => {
                        let packet = packet.respond_empty();
                        Box::pin(ready(packet))
                        // continue;
                    }
                    super::router::HandleError::Decoding(err) => {
                        continue;
                    }
                }
            }
        };
        let packet = s.await;

        debug!("WRITE: {:?}", packet);

        let mut reader = TdfReader::new(&packet.body);
        let mut out = String::new();

        out.push_str("{\n");

        // Stringify the content or append error instead
        if let Err(err) = reader.stringify(&mut out) {
            // return Ok("Failed to decode".to_string());
        }

        if out.len() == 2 {
            // Remove new line if nothing else was appended
            out.pop();
        }

        out.push('}');

        debug!("Content:{}\n\n", out);

        session.io.send(packet).await.unwrap();
    }
}
struct EmptyData;

impl Encodable for EmptyData {
    fn encode(&self, writer: &mut blaze_pk::writer::TdfWriter) {}
}

impl Decodable for EmptyData {
    fn decode(reader: &mut blaze_pk::reader::TdfReader) -> blaze_pk::error::DecodeResult<Self> {
        Ok(EmptyData)
    }
}

async fn pre_auth(state: &mut Session, req: EmptyData) -> PreAuthResponse {
    info!("pre auth request");
    PreAuthResponse {
        target: state.host_target.clone(),
    }
}

async fn ping(_state: &mut Session, req: EmptyData) -> PingResponse {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    PingResponse { time }
}

async fn auth(state: &mut Session, req: AuthRequest) -> AuthResponse {
    debug!("Auth request");
    state.io.send(Packet::notify(30722, 8, AuthNotify)).await;
    AuthResponse
}

async fn cfg(state: &mut Session, req: ClientConfigReq) -> ClientConfigRes {
    debug!("Cfg request");
    ClientConfigRes
}

async fn keep_alive(_state: &mut Session, req: EmptyData) -> EmptyData {
    EmptyData
}

async fn post_auth(_state: &mut Session, req: EmptyData) -> PostAuthRes {
    PostAuthRes
}

#[test]
fn test() {
    let s = "\x17CON".as_bytes();
    println!("{}", s[0])
}
