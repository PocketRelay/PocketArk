use super::{
    components,
    models::user_sessions::{
        IpPairAddress, NetData, NetworkAddress, QosNetworkData, UserAdded, UserUpdated,
    },
    packet::{Packet, PacketFlags},
    router::HandleError,
};
use crate::{
    blaze::packet::PacketDebug,
    database::entity::User,
    http::middleware::upgrade::UpgradedTarget,
    services::game::{manager::GetGameMessage, GameID, Player, RemovePlayerMessage, RemoveReason},
    state::App,
};
use bytes::Bytes;
use interlink::prelude::*;
use log::{debug, error};
use std::io;
use tdf::{serialize_vec, TdfSerialize};
use uuid::Uuid;

pub struct Session {
    pub uuid: Uuid,
    pub writer: SinkLink<Packet>,
    pub host_target: UpgradedTarget,
    pub user: User,
    pub net: NetData,
    pub game: Option<u32>,
}

pub type SessionLink = Link<Session>;

#[derive(Message)]
#[msg(rtype = "User")]
pub struct GetUserMessage;

impl Handler<GetUserMessage> for Session {
    type Response = Mr<GetUserMessage>;
    fn handle(&mut self, msg: GetUserMessage, ctx: &mut ServiceContext<Self>) -> Self::Response {
        Mr(self.user.clone())
    }
}

#[derive(Message)]
#[msg(rtype = "Player")]
pub struct GetPlayerMessage;

impl Handler<GetPlayerMessage> for Session {
    type Response = Mr<GetPlayerMessage>;
    fn handle(&mut self, msg: GetPlayerMessage, ctx: &mut ServiceContext<Self>) -> Self::Response {
        let player = Player::new(self.uuid, self.user.clone(), ctx.link(), self.net.clone());
        Mr(player)
    }
}

impl Service for Session {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) {
        debug!("Session started {}", &self.uuid);
    }

    fn stopping(&mut self) {
        debug!("Session stopped {}", &self.uuid);
        if let Some(game_id) = self.game {
            let user_id = self.user.id;

            tokio::spawn(async move {
                let services = App::services();

                let game = match services.games.send(GetGameMessage { game_id }).await {
                    Ok(Some(value)) => value,
                    _ => return,
                };

                let _ = game
                    .send(RemovePlayerMessage {
                        user_id,
                        reason: RemoveReason::ServerConnectionLost,
                    })
                    .await;
            });
        }
    }
}

impl Session {
    pub fn new(writer: SinkLink<Packet>, host_target: UpgradedTarget, user: User) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            writer,
            host_target,
            user,
            net: NetData::default(),
            game: None,
        }
    }

    pub fn push(&mut self, mut packet: Packet) {
        // sent as premsg for all notifys
        //  "CNTX": 1053382590009, session id
        //  "ERRC": 0, error code
        // "MADR": { (group) unknown
        // },

        if packet.header.flags.contains(PacketFlags::FLAG_NOTIFY) {
            let msg = NotifyContext {
                uid: self.user.id,
                error: 0,
            };
            packet.pre_msg = Bytes::from(serialize_vec(&msg));
        }

        self.debug_log_packet("Queued Write", &packet);
        if self.writer.sink(packet).is_err() {
            // TODO: Handle failing to send contents to writer
        }
    }

    pub fn debug_log_packet(&self, dir: &str, packet: &Packet) {
        let out = PacketDebug {
            packet,
            minified: false,
        };
        debug!("{}:\n {:?}", dir, out);
    }
}

pub struct NotifyContext {
    pub uid: u32,
    pub error: u32,
}

impl TdfSerialize for NotifyContext {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_owned(b"CNTX", self.uid);
        w.tag_owned(b"CNTX", self.error);
        w.tag_group_empty(b"MADR");
    }
}

impl StreamHandler<io::Result<Packet>> for Session {
    fn handle(&mut self, msg: io::Result<Packet>, ctx: &mut ServiceContext<Self>) {
        if let Ok(packet) = msg {
            self.debug_log_packet("Read", &packet);
            let mut addr = ctx.link();
            tokio::spawn(async move {
                let router = App::router();
                let response = match router.handle(&mut addr, packet) {
                    // Await the handler response future
                    Ok(fut) => fut.await,

                    // Handle any errors that occur
                    Err(err) => {
                        match err {
                            // No handler set-up just respond with a default empty response
                            HandleError::MissingHandler(packet) => packet.respond_empty(),
                            HandleError::Decoding(packet, err) => {
                                error!(
                                    "Error while decoding packet ({:?}): {:?}",
                                    packet.header, err
                                );
                                return;
                            }
                        }
                    }
                };
                // Push the response to the client
                addr.push(response);
            });
        } else {
            ctx.stop();
        }
    }
}

impl ErrorHandler<io::Error> for Session {
    fn handle(&mut self, _err: io::Error, _ctx: &mut ServiceContext<Self>) -> ErrorAction {
        ErrorAction::Continue
    }
}

/// Extension for links to push packets for session links
pub trait PushExt {
    fn push(&self, packet: Packet);
}

impl PushExt for Link<Session> {
    fn push(&self, packet: Packet) {
        let _ = self.do_send(WriteMessage(packet));
    }
}

#[derive(Message)]
pub struct WriteMessage(pub Packet);

impl Handler<WriteMessage> for Session {
    type Response = ();

    fn handle(&mut self, msg: WriteMessage, _ctx: &mut ServiceContext<Self>) -> Self::Response {
        self.push(msg.0);
    }
}

#[derive(Message)]
pub struct NetworkInfoMessage {
    pub addr: NetworkAddress,
    pub qos: QosNetworkData,
}

impl Handler<NetworkInfoMessage> for Session {
    type Response = ();

    fn handle(
        &mut self,
        msg: NetworkInfoMessage,
        ctx: &mut ServiceContext<Self>,
    ) -> Self::Response {
        self.net.addr = msg.addr;
        self.net.qos = msg.qos;
        let _ = ctx.shared_link().do_send(UpdateUserMessage);
    }
}

#[derive(Message)]
pub struct HardwareFlagsMessage {
    pub flags: u8,
}

impl Handler<HardwareFlagsMessage> for Session {
    type Response = ();

    fn handle(
        &mut self,
        msg: HardwareFlagsMessage,
        ctx: &mut ServiceContext<Self>,
    ) -> Self::Response {
        self.net.hwfg = msg.flags;
        let _ = ctx.shared_link().do_send(UpdateUserMessage);
    }
}

#[derive(Message)]
pub struct UpdateUserMessage;

impl Handler<UpdateUserMessage> for Session {
    type Response = ();

    fn handle(
        &mut self,
        _msg: UpdateUserMessage,
        _ctx: &mut ServiceContext<Self>,
    ) -> Self::Response {
        self.push(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::NOTIFY_USER_UPDATED,
            UserUpdated {
                player_id: self.user.id,
                game_id: self.game,
                net_data: self.net.clone(),
            },
        ));
    }
}

#[derive(Message)]
#[msg(rtype = "UpgradedTarget")]
pub struct GetHostTarget;

impl Handler<GetHostTarget> for Session {
    type Response = Mr<GetHostTarget>;

    fn handle(&mut self, _msg: GetHostTarget, _ctx: &mut ServiceContext<Self>) -> Self::Response {
        Mr(self.host_target.clone())
    }
}

#[derive(Message)]
pub struct UserAddedMessage;

impl Handler<UserAddedMessage> for Session {
    type Response = ();

    fn handle(
        &mut self,
        _msg: UserAddedMessage,
        _ctx: &mut ServiceContext<Self>,
    ) -> Self::Response {
        self.push(Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::NOTIFY_USER_ADDED,
            UserAdded {
                player_id: self.user.id,
                name: self.user.username.to_string(),
                game_id: self.game,
                net_data: self.net.clone(),
            },
        ));
    }
}

#[derive(Message)]
pub struct InformSessions {
    /// The link to send the set session to
    pub links: Vec<Link<Session>>,
}

impl Handler<InformSessions> for Session {
    type Response = ();

    fn handle(&mut self, msg: InformSessions, _ctx: &mut ServiceContext<Self>) -> Self::Response {
        let packet = Packet::notify(
            components::user_sessions::COMPONENT,
            components::user_sessions::NOTIFY_USER_ADDED,
            UserAdded {
                player_id: self.user.id,
                name: self.user.username.to_string(),
                game_id: self.game,
                net_data: self.net.clone(),
            },
        );
        for link in msg.links {
            link.push(packet.clone());
        }
    }
}

#[derive(Message)]
pub struct SetGameMessage {
    pub game: Option<GameID>,
}

impl Handler<SetGameMessage> for Session {
    type Response = ();

    fn handle(&mut self, msg: SetGameMessage, ctx: &mut ServiceContext<Self>) {
        self.game = msg.game;

        let _ = ctx.shared_link().do_send(UpdateUserMessage);
    }
}
