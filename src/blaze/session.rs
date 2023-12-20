use super::{
    components::{self, user_sessions},
    models::{
        game_manager::RemoveReason,
        user_sessions::{
            HardwareFlags, IpPairAddress, NetworkAddress, NotifyUserAdded, NotifyUserRemoved,
            NotifyUserUpdated, QosNetworkData, UserDataFlags, UserIdentification,
            UserSessionExtendedData, UserSessionExtendedDataUpdate,
        },
    },
    packet::{FrameFlags, Packet, PacketCodec},
    router::BlazeRouter,
};
use crate::{
    blaze::packet::PacketDebug,
    database::entity::{users::UserId, User},
    services::{
        game::{GameID, Player, WeakGameRef},
        sessions::Sessions,
    },
    utils::lock::{QueueLock, QueueLockGuard, TicketAquireFuture},
};
use bytes::Bytes;
use futures::{
    future::BoxFuture,
    stream::{SplitSink, SplitStream},
    Sink, SinkExt, Stream, StreamExt,
};
use hyper::upgrade::Upgraded;
use log::{debug, error, warn};
use parking_lot::Mutex;
use serde::Serialize;
use std::{
    future::Future,
    pin::Pin,
    sync::Weak,
    task::{Context, Poll},
};
use std::{io, sync::Arc, task::ready};
use tdf::{serialize_vec, TdfSerialize};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinSet,
};
use tokio_util::codec::Framed;
use uuid::Uuid;

pub type SessionLink = Arc<Session>;
pub type WeakSessionLink = Weak<Session>;

pub struct Session {
    pub uuid: Uuid,

    busy_lock: QueueLock,
    tx: mpsc::UnboundedSender<Packet>,

    pub data: Mutex<SessionExtData>,
    // Add when session service implemented:
    sessions: Arc<Sessions>,
}

#[derive(Clone)]
pub struct SessionNotifyHandle {
    busy_lock: QueueLock,
    tx: mpsc::UnboundedSender<Packet>,
}

impl SessionNotifyHandle {
    /// Pushes a new notification packet, this will aquire a queue position
    /// waiting until the current response is handled before sending
    pub fn notify(&self, packet: Packet) {
        let tx = self.tx.clone();
        let busy_lock = self.busy_lock.aquire();
        tokio::spawn(async move {
            let _guard = busy_lock.await;
            let _ = tx.send(packet);
        });
    }
}

pub struct SessionExtData {
    pub user: Arc<User>,
    pub net: Arc<NetData>,
    game: Option<SessionGameData>,
    subscribers: Vec<(UserId, SessionNotifyHandle)>,
}

struct SessionGameData {
    game_id: GameID,
    game_ref: WeakGameRef,
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
            game: self.game.as_ref().map(|game| game.game_id),
            user_id: self.user.id,
        }
    }

    fn add_subscriber(&mut self, user_id: UserId, subscriber: SessionNotifyHandle) {
        // Notify the addition of this user data to the subscriber
        subscriber.notify(Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_ADDED,
            NotifyUserAdded {
                session_data: self.ext(),
                user: UserIdentification::from_user(&self.user),
            },
        ));

        // Notify the user that they are now subscribed to this user
        subscriber.notify(Packet::notify(
            user_sessions::COMPONENT,
            user_sessions::USER_SESSION_EXTENDED_DATA_UPDATE,
            NotifyUserUpdated {
                flags: UserDataFlags::SUBSCRIBED | UserDataFlags::ONLINE,
                user_id: self.user.id,
            },
        ));

        self.subscribers.push((user_id, subscriber));
    }

    fn remove_subscriber(&mut self, user_id: UserId) {
        let subscriber = self
            .subscribers
            .iter()
            // Find the subscriber to remove
            .position(|(id, _sub)| user_id.eq(id))
            // Remove the subscriber
            .map(|index| self.subscribers.swap_remove(index));

        if let Some((_, subscriber)) = subscriber {
            // Notify the subscriber they've removed the user subcription
            subscriber.notify(Packet::notify(
                user_sessions::COMPONENT,
                user_sessions::USER_REMOVED,
                NotifyUserRemoved { user_id },
            ))
        }
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

        self.subscribers
            .iter()
            .for_each(|(_, sub)| sub.notify(packet.clone()));
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

impl Session {
    pub async fn start(
        io: Upgraded,
        user: User,
        router: Arc<BlazeRouter>,
        sessions: Arc<Sessions>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();

        let user_id = user.id;

        let session = Arc::new(Self {
            uuid: Uuid::new_v4(),
            busy_lock: QueueLock::new(),
            tx,
            data: Mutex::new(SessionExtData::new(user)),
            sessions,
        });

        // Add the session to the sessions service
        let weak_link = Arc::downgrade(&session);
        session.sessions.add_session(user_id, weak_link);

        debug!("Session started {}", &session.uuid);

        SessionFuture {
            io: Framed::new(io, PacketCodec),
            router: &router,
            rx,
            session: session.clone(),
            read_state: ReadState::Recv,
            write_state: WriteState::Recv,
            stop: false,
        }
        .await;

        session.stop();
    }

    pub fn notify_handle(&self) -> SessionNotifyHandle {
        SessionNotifyHandle {
            busy_lock: self.busy_lock.clone(),
            tx: self.tx.clone(),
        }
    }

    /// Internal session stopped function called by the reader when
    /// the connection is terminated, cleans up any references and
    /// asserts only 1 strong reference exists
    fn stop(self: Arc<Self>) {
        // Clear authentication
        self.clear_player();

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

    /// Clears the current game returning the game data if
    /// the player was in a game
    ///
    /// Called by the game itself when the player has been removed
    pub fn clear_game(&self) -> Option<(UserId, WeakGameRef)> {
        let mut game: Option<SessionGameData> = None;
        let mut user_id: Option<UserId> = None;

        self.update_data(|data| {
            game = data.game.take();
            user_id = Some(data.user.id);
        });

        let game = game?;
        let user_id = user_id?;

        Some((user_id, game.game_ref))
    }

    /// Called to remove the player from its current game
    pub fn remove_from_game(&self) {
        let (player_id, game_ref) = match self.clear_game() {
            Some(value) => value,
            // Player isn't in a game
            None => return,
        };

        let game_ref = match game_ref.upgrade() {
            Some(value) => value,
            // Game doesn't exist anymore
            None => return,
        };

        // Spawn an async task to handle removing the player
        tokio::spawn(async move {
            let game = &mut *game_ref.write().await;
            game.remove_player(player_id, RemoveReason::PlayerLeft);
        });
    }

    pub fn clear_player(&self) {
        self.remove_from_game();

        // Check that theres authentication
        let data = &mut *self.data.lock();

        // Existing sessions must be unsubscribed
        data.subscribers.clear();

        self.sessions.remove_session(data.user.id);
    }

    #[inline]
    pub fn add_subscriber(&self, user_id: UserId, subscriber: SessionNotifyHandle) {
        self.data.lock().add_subscriber(user_id, subscriber);
    }

    #[inline]
    pub fn remove_subscriber(&self, user_id: UserId) {
        self.data.lock().remove_subscriber(user_id);
    }

    #[inline]
    pub fn set_hardware_flags(&self, value: HardwareFlags) {
        self.update_data(|data| {
            data.net = Arc::new(data.net.with_hardware_flags(value));
        });
    }

    #[inline]
    pub fn set_network_info(&self, address: NetworkAddress, qos: QosNetworkData) {
        self.update_data(|data| {
            data.net = Arc::new(data.net.with_basic(address, qos));
        });
    }

    #[inline]
    fn update_data<F>(&self, update: F)
    where
        F: FnOnce(&mut SessionExtData),
    {
        let data = &mut *self.data.lock();
        update(data);
        data.publish_update();
    }

    pub fn set_game(&self, game_id: GameID, game_ref: WeakGameRef) {
        // Remove the player from the game if they are already present in one
        self.remove_from_game();

        // Set the current game
        self.update_data(|data| {
            data.game = Some(SessionGameData { game_id, game_ref });
        });
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

/// Future for processing a session
struct SessionFuture<'a> {
    /// The IO for reading and writing
    io: Framed<Upgraded, PacketCodec>,
    /// Receiver for packets to write
    rx: mpsc::UnboundedReceiver<Packet>,
    /// The session this link is for
    session: SessionLink,
    /// The router to use
    router: &'a BlazeRouter,
    /// The reading state
    read_state: ReadState<'a>,
    /// The writing state
    write_state: WriteState,
    /// Whether the future has been stopped
    stop: bool,
}

/// Session future writing state
enum WriteState {
    /// Waiting for a packet to write
    Recv,
    /// Waiting for the framed to become read
    Write { packet: Option<Packet> },
    /// Flushing the framed
    Flush,
}

/// Session future reading state
enum ReadState<'a> {
    /// Waiting for a packet
    Recv,
    /// Aquiring a lock guard
    Aquire {
        /// Future for the locking guard
        ticket: TicketAquireFuture,
        /// The packet that was read
        packet: Option<Packet>,
    },
    /// Future for a handler is being polled
    Handle {
        /// Locking guard
        guard: QueueLockGuard,
        /// Handle future
        future: BoxFuture<'a, Packet>,
    },
}

impl SessionFuture<'_> {
    /// Polls the write state, the poll ready state returns whether
    /// the future should continue
    fn poll_write_state(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        match &mut self.write_state {
            WriteState::Recv => {
                // Try receive a packet from the write channel
                let result = ready!(Pin::new(&mut self.rx).poll_recv(cx));

                if let Some(packet) = result {
                    self.write_state = WriteState::Write {
                        packet: Some(packet),
                    };
                } else {
                    // All writers have closed, session must be closed (Future end)
                    self.stop = true;
                }
            }
            WriteState::Write { packet } => {
                // Wait until the inner is ready
                if ready!(Pin::new(&mut self.io).poll_ready(cx)).is_ok() {
                    let mut packet = packet
                        .take()
                        .expect("Unexpected write state without packet");

                    self.session.debug_log_packet("Send", &packet);

                    // TODO: MOVE THIS ELSEWHERE
                    {
                        // sent as premsg for all notifys
                        //  "CNTX": 1053382590009, session id
                        //  "ERRC": 0, error code
                        // "MADR": { (group) unknown
                        // },

                        // TODO: Notify context may need to be appended elsewhere instead
                        if packet.frame.flags.contains(FrameFlags::FLAG_NOTIFY) {
                            let uid = {
                                let data = &*self.session.data.lock();
                                data.user.id
                            };

                            let msg = NotifyContext { uid, error: 0 };
                            packet.pre_msg = Bytes::from(serialize_vec(&msg));
                        }
                    }

                    // Write the packet to the buffer
                    Pin::new(&mut self.io)
                        .start_send(packet)
                        // Packet encoder impl shouldn't produce errors
                        .expect("Packet encoder errored");

                    self.write_state = WriteState::Flush;
                } else {
                    // Failed to ready, session must be closed
                    self.stop = true;
                }
            }
            WriteState::Flush => {
                // Wait until the flush is complete
                if ready!(Pin::new(&mut self.io).poll_flush(cx)).is_ok() {
                    self.write_state = WriteState::Recv;
                } else {
                    // Failed to flush, session must be closed
                    self.stop = true
                }
            }
        }

        Poll::Ready(())
    }

    /// Polls the read state, the poll ready state returns whether
    /// the future should continue
    fn poll_read_state(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        match &mut self.read_state {
            ReadState::Recv => {
                // Try receive a packet from the write channel
                let result = ready!(Pin::new(&mut self.io).poll_next(cx));

                if let Some(Ok(packet)) = result {
                    let ticket = self.session.busy_lock.aquire();
                    self.read_state = ReadState::Aquire {
                        ticket,
                        packet: Some(packet),
                    }
                } else {
                    // Reader has closed or reading encountered an error (Either way stop reading)
                    self.stop = true;
                }
            }
            ReadState::Aquire { ticket, packet } => {
                let guard = ready!(Pin::new(ticket).poll(cx));
                let packet = packet
                    .take()
                    .expect("Unexpected aquire state without packet");

                self.session.debug_log_packet("Receive", &packet);

                let future = self.router.handle(self.session.clone(), packet);

                // Move onto a handling state
                self.read_state = ReadState::Handle { guard, future };
            }
            ReadState::Handle {
                guard: _gaurd,
                future,
            } => {
                // Poll the handler until completion
                let response = ready!(Pin::new(future).poll(cx));

                // Send the response to the writer
                _ = self.session.tx.send(response);

                // Reset back to the reading state
                self.read_state = ReadState::Recv;
            }
        }
        Poll::Ready(())
    }
}

impl Future for SessionFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        while this.poll_write_state(cx).is_ready() {}
        while this.poll_read_state(cx).is_ready() {}

        if this.stop {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
