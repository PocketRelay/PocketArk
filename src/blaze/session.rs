use super::{
    components::{self, user_sessions},
    models::user_sessions::{
        HardwareFlags, IpPairAddress, NetworkAddress, NotifyUserAdded, NotifyUserRemoved,
        NotifyUserUpdated, QosNetworkData, UserDataFlags, UserIdentification,
        UserSessionExtendedData, UserSessionExtendedDataUpdate,
    },
    packet::{Packet, PacketCodec, PacketFlags},
    router::BlazeRouter,
};
use crate::{
    blaze::packet::PacketDebug,
    database::entity::{users::UserId, User},
    http::middleware::upgrade::UpgradedTarget,
    services::game::{manager::GetGameMessage, GameID, Player, RemovePlayerMessage, RemoveReason},
    state::App,
};
use bytes::Bytes;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use hyper::upgrade::Upgraded;
use interlink::prelude::*;
use log::{debug, error, warn};
use serde::Serialize;
use std::{io, sync::Arc};
use tdf::{serialize_vec, TdfSerialize};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinSet,
};
use tokio_util::codec::Framed;
use uuid::Uuid;

pub struct Session {
    pub uuid: Uuid,
    writer: mpsc::UnboundedSender<WriteMessage>,
    pub host_target: UpgradedTarget,
    pub data: RwLock<SessionExtData>,

    router: Arc<BlazeRouter>,
}

pub struct SessionExtData {
    pub user: Arc<User>,
    pub net: Arc<NetData>,
    pub game: Option<GameID>,
    subscribers: Vec<(UserId, SessionLink)>,
}

impl SessionExtData {
    pub fn new(user: User) -> Self {
        Self {
            user: Arc::new(user),
            net: Default::default(),
            game: Default::default(),
            subscribers: Default::default(),
        }
    }

    fn ext(&self) -> UserSessionExtendedData {
        UserSessionExtendedData {
            net: self.net.clone(),
            game: self.game,
            user_id: self.user.id,
        }
    }

    fn add_subscriber(&mut self, user_id: UserId, subscriber: SessionLink) {
        // Create the details packets
        let added_notify = Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_ADDED,
            NotifyUserAdded {
                session_data: self.ext(),
                user: UserIdentification::from_user(&self.user),
            },
        );

        // Create update notifying the user of the subscription
        let update_notify = Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_SESSION_EXTENDED_DATA_UPDATE,
            NotifyUserUpdated {
                flags: UserDataFlags::SUBSCRIBED | UserDataFlags::ONLINE,
                user_id: self.user.id,
            },
        );

        self.subscribers.push((user_id, subscriber.clone()));
        subscriber.push(added_notify);
        subscriber.push(update_notify);
    }

    fn remove_subscriber(&mut self, user_id: UserId) {
        let index = match self.subscribers.iter().position(|(id, _)| user_id.eq(id)) {
            Some(value) => value,
            None => return,
        };

        let (_, subscriber) = self.subscribers.swap_remove(index);

        // Create the details packets
        let removed_notify = Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_REMOVED,
            NotifyUserRemoved { user_id },
        );

        subscriber.push(removed_notify)
    }

    /// Publishes changes of the session data to all the
    /// subscribed session links
    fn publish_update(&self) {
        let packet = Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_SESSION_EXTENDED_DATA_UPDATE,
            UserSessionExtendedDataUpdate {
                user_id: self.user.id,
                data: self.ext(),
                subs: self.subscribers.len(),
            },
        );

        for (_, subscriber) in &self.subscribers {
            subscriber.push(packet.clone());
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct NetData {
    pub addr: NetworkAddress,
    pub qos: QosNetworkData,
    pub hardware_flags: HardwareFlags,
}

impl NetData {
    // Re-creates the current net data using the provided address and QOS data
    pub fn with_basic(&self, addr: NetworkAddress, qos: QosNetworkData) -> Self {
        Self {
            addr,
            qos,
            hardware_flags: self.hardware_flags,
        }
    }

    /// Re-creates the current net data using the provided hardware flags
    pub fn with_hardware_flags(&self, flags: HardwareFlags) -> Self {
        Self {
            addr: self.addr.clone(),
            qos: self.qos,
            hardware_flags: flags,
        }
    }
}

// Writer for writing packets
struct SessionWriter {
    inner: SplitSink<Framed<Upgraded, PacketCodec>, Packet>,
    rx: mpsc::UnboundedReceiver<WriteMessage>,
    link: SessionLink,
}

pub enum WriteMessage {
    Write(Packet),
    Close,
}

impl SessionWriter {
    pub async fn process(mut self) {
        while let Some(msg) = self.rx.recv().await {
            let packet = match msg {
                WriteMessage::Write(packet) => packet,
                WriteMessage::Close => break,
            };

            self.link.debug_log_packet("Queued Write", &packet);
            if self.inner.send(packet).await.is_err() {
                break;
            }
        }
    }
}

struct SessionReader {
    inner: SplitStream<Framed<Upgraded, PacketCodec>>,
    link: SessionLink,
}

impl SessionReader {
    pub async fn process(mut self) {
        let mut tasks = JoinSet::new();

        while let Some(Ok(packet)) = self.inner.next().await {
            let link = self.link.clone();
            tasks.spawn(async move {
                link.debug_log_packet("Read", &packet);
                let response = match link.router.handle(link.clone(), packet) {
                    // Await the handler response future
                    Ok(fut) => fut.await,

                    // Handle no handler for packet
                    Err(packet) => {
                        debug!("Missing packet handler");
                        Packet::response_empty(&packet)
                    }
                };
                // Push the response to the client
                link.push(response);
            });
        }

        tasks.shutdown().await;

        self.link.stop().await;
    }
}

pub type SessionLink = Arc<Session>;

impl Session {
    pub fn start(io: Upgraded, host_target: UpgradedTarget, user: User, router: Arc<BlazeRouter>) {
        let framed = Framed::new(io, PacketCodec);
        let (write, read) = framed.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let session = Arc::new(Self {
            uuid: Uuid::new_v4(),
            writer: tx,
            host_target,
            data: RwLock::new(SessionExtData::new(user)),
            router,
        });

        debug!("Session started {}", &session.uuid);

        let reader = SessionReader {
            link: session.clone(),
            inner: read,
        };

        let writer = SessionWriter {
            link: session.clone(),
            rx,
            inner: write,
        };

        tokio::spawn(reader.process());
        tokio::spawn(writer.process());
    }

    /// Internal session stopped function called by the reader when
    /// the connection is terminated, cleans up any references and
    /// asserts only 1 strong reference exists
    async fn stop(self: Arc<Self>) {
        // Tell the write half to close and wait until its closed
        _ = self.writer.send(WriteMessage::Close);
        self.writer.closed().await;

        // Clear authentication
        self.clear_player().await;

        let session: Self = match Arc::try_unwrap(self) {
            Ok(value) => value,
            Err(arc) => {
                let references = Arc::strong_count(&arc);
                warn!(
                    "Session {} was stopped but {} references to it still exist",
                    arc.uuid, references
                );
                return;
            }
        };

        debug!("Session stopped (SID: {})", session.uuid);
    }

    pub async fn clear_player(&self) {
        // Check that theres authentication
        let data = &mut *self.data.write().await;

        // Existing sessions must be unsubscribed
        data.subscribers.clear();

        // Remove session from games service
        if let Some(game_id) = data.game {
            let user_id = data.user.id;

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
        }

        // Remove the session from the sessions service
        // self.sessions.remove_session(data.player.id).await;
    }

    pub async fn add_subscriber(&self, user_id: UserId, subscriber: SessionLink) {
        let data = &mut *self.data.write().await;
        data.add_subscriber(user_id, subscriber);
    }

    pub async fn remove_subscriber(&self, user_id: UserId) {
        let data = &mut *self.data.write().await;
        data.remove_subscriber(user_id);
    }

    pub async fn set_hardware_flags(&self, value: HardwareFlags) {
        let data = &mut *self.data.write().await;

        data.net = Arc::new(data.net.with_hardware_flags(value));
        data.publish_update();
    }

    pub async fn set_network_info(&self, address: NetworkAddress, qos: QosNetworkData) {
        let data = &mut *self.data.write().await;

        data.net = Arc::new(data.net.with_basic(address, qos));
        data.publish_update();
    }

    pub async fn set_game(&self, game: Option<GameID>) {
        let data = &mut *self.data.write().await;

        data.game = game;
        data.publish_update();
    }

    pub fn push(&self, mut packet: Packet) {
        // sent as premsg for all notifys
        //  "CNTX": 1053382590009, session id
        //  "ERRC": 0, error code
        // "MADR": { (group) unknown
        // },

        // TODO: Notify context may need to be appended elsewhere instead
        // if packet.header.flags.contains(PacketFlags::FLAG_NOTIFY) {
        //     let msg = NotifyContext {
        //         uid: self.user.id,
        //         error: 0,
        //     };
        //     packet.pre_msg = Bytes::from(serialize_vec(&msg));
        // }

        self.debug_log_packet("Queued Write", &packet);
        _ = self.writer.send(WriteMessage::Write(packet))
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
