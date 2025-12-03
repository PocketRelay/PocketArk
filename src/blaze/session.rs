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
    blaze::{
        blaze_socket::{BlazeLock, BlazeLockFuture, BlazeRx, BlazeSocketFuture, BlazeTx},
        components::component_key,
        data::SessionData,
        packet::PacketDebug,
    },
    database::entity::{User, users::UserId},
    services::{
        game::{GameID, WeakGameRef},
        sessions::Sessions,
    },
    utils::lock::{QueueLock, QueueLockGuard, TicketAcquireFuture},
};
use bytes::Bytes;
use futures::{
    Sink, SinkExt, Stream, StreamExt,
    future::BoxFuture,
    stream::{SplitSink, SplitStream},
};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use log::{debug, error, log_enabled, warn};
use parking_lot::Mutex;
use serde::Serialize;
use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::Weak,
    task::{Context, Poll},
};
use std::{io, sync::Arc, task::ready};
use tdf::{TdfSerialize, serialize_vec};
use tokio::{
    spawn,
    sync::{RwLock, mpsc},
    task::JoinSet,
};
use tokio_util::codec::Framed;
use uuid::Uuid;

pub type SessionLink = Arc<Session>;
pub type WeakSessionLink = Weak<Session>;

pub struct Session {
    pub id: Uuid,

    /// Handle for sending packets to this session
    pub tx: BlazeTx,

    /// Data associated with the session
    pub data: SessionData,
}

impl Session {
    pub fn start(
        id: Uuid,
        io: Upgraded,
        data: SessionData,
        router: Arc<BlazeRouter>,
    ) -> WeakSessionLink {
        // Create blaze socket handler
        let (blaze_future, blaze_rx, blaze_tx) =
            BlazeSocketFuture::new(Framed::new(TokioIo::new(io), PacketCodec::default()));

        spawn(async move {
            if let Err(cause) = blaze_future.await {
                error!("error running blaze socket future: {cause:?}")
            }

            debug!("session blaze future completed");
        });

        debug!("session started (SID: {id})");

        // Create session handler
        let session = Arc::new(Self {
            id,
            tx: blaze_tx,
            data,
        });

        let weak_session = Arc::downgrade(&session);

        spawn({
            let session = session;

            async move {
                // Run the session to completion
                SessionFuture::new(blaze_rx, &session, &router).await;

                debug!("session future complete");

                // Clear session data, speeds up process of ending the session
                // prevents session data being accessed while shutting down
                session.data.clear_auth();

                debug!("session auth cleared, session dropped");
            }
        });

        weak_session
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

struct DebugSessionData {
    id: Uuid,
    auth: Option<Arc<User>>,
    action: &'static str,
}

impl Debug for DebugSessionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Session ({}): {}", self.id, self.action)?;

        if let Some(auth) = &self.auth {
            writeln!(f, "Auth ({}): (Name: {})", auth.id, &auth.username)?;
        }

        Ok(())
    }
}

/// Future for processing a session
struct SessionFuture<'a> {
    /// Receiver for packets to handle
    rx: BlazeRx,
    /// The session this link is for
    session: &'a SessionLink,
    /// The router to use
    router: &'a BlazeRouter,
    /// State of the future
    state: SessionFutureState<'a>,
}

/// Session future reading state
enum SessionFutureState<'a> {
    /// Waiting for inbound packet
    Accept,
    /// Waiting to acquire write handling lock
    Acquire {
        /// Future for the locking guard
        lock_future: BlazeLockFuture,
        /// The packet that was read
        packet: Option<Packet>,
    },
    /// Future for a handler is being polled
    Handle {
        /// Access to the sender for sending the response
        tx: BlazeLock,
        /// Handle future
        future: BoxFuture<'a, Packet>,
    },
}

impl SessionFuture<'_> {
    pub fn new<'a>(
        rx: BlazeRx,
        session: &'a Arc<Session>,
        router: &'a BlazeRouter,
    ) -> SessionFuture<'a> {
        SessionFuture {
            router,
            rx,
            session,
            state: SessionFutureState::Accept,
        }
    }
}

impl Future for SessionFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            // Poll checking if the connection has timed-out
            if this.session.data.poll_keep_alive_dead(cx) {
                return Poll::Ready(());
            }

            match &mut this.state {
                SessionFutureState::Accept => {
                    let packet = match ready!(this.rx.poll_recv(cx)) {
                        Some(value) => value,
                        None => {
                            // Read half of the socket has terminated, nothing left to handle
                            return Poll::Ready(());
                        }
                    };

                    // Acquire a write lock future (Reserve our space for sending the response)
                    let lock_future = Box::pin(this.session.tx.acquire_tx());

                    this.state = SessionFutureState::Acquire {
                        lock_future,
                        packet: Some(packet),
                    }
                }
                SessionFutureState::Acquire {
                    lock_future,
                    packet,
                } => {
                    let guard = ready!(Pin::new(lock_future).poll(cx));
                    let packet = packet
                        .take()
                        .expect("Unexpected acquire state without packet");

                    debug_log_packet(this.session, "Receive", &packet);

                    let future = this.router.handle(this.session.clone(), packet);

                    // Move onto a handling state
                    this.state = SessionFutureState::Handle { tx: guard, future };
                }
                SessionFutureState::Handle { tx, future } => {
                    // Poll the handler until completion
                    let response = ready!(Pin::new(future).poll(cx));

                    // Send the response to the writer
                    if tx.send(response).is_err() {
                        // Write half has closed, cease reading
                        return Poll::Ready(());
                    }

                    // Reset back to the reading state
                    this.state = SessionFutureState::Accept;
                }
            }
        }
    }
}

/// Logs debugging information about a player
fn debug_log_packet(session: &Session, action: &'static str, packet: &Packet) {
    // Skip if debug logging is disabled
    if !log_enabled!(log::Level::Debug) {
        return;
    }

    let key = component_key(packet.frame.component, packet.frame.command);

    // // Don't log the packet if its debug ignored
    // if DEBUG_IGNORED_PACKETS.contains(&key) {
    //     return;
    // }

    let id = session.id;
    let auth = session.data.get_player();

    let debug_data = DebugSessionData { action, id, auth };
    let debug_packet = PacketDebug {
        packet,
        minified: false,
    };

    debug!("\n{debug_data:?}{debug_packet:?}");
}
