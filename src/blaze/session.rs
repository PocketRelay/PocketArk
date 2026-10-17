use crate::blaze::data::{SessionData, SessionInitData};
use futures::{TryFutureExt, future::BoxFuture};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use pocket_ark_blaze_networking::{
    blaze_socket::{BlazeLock, BlazeLockFuture, BlazeRx, BlazeSocketFuture, BlazeTx},
    debug_packet,
    packet::{Packet, PacketCodec, PacketDebug},
    router::BlazeRouter,
};
use pocket_ark_database::dto::users::UserDto;
use std::{
    future::Future,
    pin::Pin,
    sync::Weak,
    task::{Context, Poll},
};
use std::{sync::Arc, task::ready};
use tdf::{TdfSerialize, types::bytes::serialize_bytes};
use tokio::spawn;
use tokio_util::codec::Framed;
use tracing::{Instrument, Span};
use uuid::Uuid;

pub type SessionLink = Arc<Session>;
pub type WeakSessionLink = Weak<Session>;

pub struct Session {
    pub id: Uuid,

    /// Handle for sending packets to this session
    tx: BlazeTx,

    /// Data associated with the session
    pub data: SessionData,
}

impl Session {
    fn new(id: Uuid, tx: BlazeTx, data: SessionData) -> Self {
        Self { id, tx, data }
    }

    #[tracing::instrument(
        name = "session",
        skip_all,
        fields(
            session_id = %id,
            user_id = %user.id,
            username = %user.username
        )
    )]
    pub fn start(
        id: Uuid,
        io: Upgraded,
        user: &UserDto,
        init_data: SessionInitData,
        router: Arc<BlazeRouter<SessionLink>>,
    ) -> WeakSessionLink {
        // Create blaze socket handler
        let (blaze_future, blaze_rx, blaze_tx) =
            BlazeSocketFuture::new(Framed::new(TokioIo::new(io), PacketCodec));

        let session_span = Span::current();
        let data = SessionData::new(init_data.addr, init_data.association_id);

        let blaze_future = blaze_future
            .inspect_err(|error| tracing::error!(?error, "error running blaze socket future"))
            .inspect_ok(|_| tracing::debug!("session blaze future completed"))
            .instrument(session_span.clone());

        // Create session handler
        let session = Arc::new(Self::new(id, blaze_tx, data));
        let weak_session = Arc::downgrade(&session);
        let session_future = session
            .run(blaze_rx, router)
            .instrument(session_span.clone());

        spawn(blaze_future);
        spawn(session_future);

        tracing::debug!("session started");
        weak_session
    }

    // Extra layer to provide borrowed references of session and router so
    // that the session future lifetimes can be qualified
    async fn run(self: Arc<Self>, rx: BlazeRx, router: Arc<BlazeRouter<SessionLink>>) {
        SessionFuture::new(rx, &self, &router).await;
        tracing::debug!("session future complete");
    }

    pub fn notify(&self, mut packet: Packet) {
        // if let Some(player) = &current_user
        //     && packet.frame.flags.contains(FrameFlags::FLAG_NOTIFY)
        // {
        //     let uid = player.id;
        //     let msg = NotifyContext { uid, error: 0 };
        //     packet.pre_msg = serialize_bytes(&msg);
        // }

        let msg = NotifyContext { uid: 1, error: 0 };
        packet.pre_msg = serialize_bytes(&msg);
        self.tx.notify(packet);
    }
}

pub struct NotifyContext {
    pub uid: u32,
    pub error: u32,
}

impl TdfSerialize for NotifyContext {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_owned(b"CNTX", self.uid);
        w.tag_owned(b"ERRC", self.error);
        w.tag_group_empty(b"MADR");
    }
}

/// Future for processing a session
///
/// Produces [Poll::Ready] when the session has completed / closed
struct SessionFuture<'a> {
    /// Receiver for packets to handle
    rx: BlazeRx,
    /// The session this link is for
    session: &'a SessionLink,
    /// The router to use
    router: &'a BlazeRouter<SessionLink>,
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
        router: &'a BlazeRouter<SessionLink>,
    ) -> SessionFuture<'a> {
        SessionFuture {
            router,
            rx,
            session,
            state: SessionFutureState::Accept,
        }
    }
}

impl Drop for SessionFuture<'_> {
    fn drop(&mut self) {
        // Clear session data, speeds up process of ending the session
        // prevents session data being accessed while shutting down
        self.session.data.clear_auth();
        tracing::debug!("session auth cleared, session dropped");
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

                    debug_packet!(&packet, direction = "request", "receiving packet");

                    let future = this.router.handle(this.session.clone(), packet);

                    // Move onto a handling state
                    this.state = SessionFutureState::Handle { tx: guard, future };
                }
                SessionFutureState::Handle { tx, future } => {
                    // Poll the handler until completion
                    let response = ready!(Pin::new(future).poll(cx));

                    debug_packet!(&response, direction = "response", "sending packet");

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
