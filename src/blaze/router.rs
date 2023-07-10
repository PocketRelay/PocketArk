//! Router implementation for routing packet components to different functions
//! and automatically decoding the packet contents to the function type

use blaze_pk::{
    codec::Encodable,
    error::{DecodeError, DecodeResult},
};
use std::{
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{ready, Context, Poll},
};

use super::packet::Packet;

/// Trait implementing by structures which can be created from a request
/// packet and is used for the arguments on routing functions
pub trait FromRequest: Sized + Send + 'static {
    /// Takes the value from the request returning a decode result of
    /// whether the value could be created
    ///
    /// `req` The request packet
    fn from_request(req: &Packet) -> DecodeResult<Self>;
}

/// Wrapper over the [FromRequest] type to support the unit type
/// to differentiate
pub trait FromRequestInternal: Sized + 'static {
    fn from_request(req: &Packet) -> DecodeResult<Self>;
}

/// Unit type implementation for handlers that don't take a req type
impl FromRequestInternal for () {
    fn from_request(_req: &Packet) -> DecodeResult<Self> {
        Ok(())
    }
}

/// Implementation for normal [FromRequest] implementations
impl<F: FromRequest> FromRequestInternal for F {
    fn from_request(req: &Packet) -> DecodeResult<Self> {
        F::from_request(req)
    }
}

/// Trait for a type that can be converted into a packet
/// response using the header from the request packet
pub trait IntoResponse: 'static {
    /// Into packet conversion
    fn into_response(self, req: &Packet) -> Packet;
}

/// Into response imeplementation for encodable responses
/// which just calls res.respond
impl<E> IntoResponse for E
where
    E: Encodable + 'static,
{
    fn into_response(self, req: &Packet) -> Packet {
        req.respond(self)
    }
}

/// Pin boxed future type that is Send and lives for 'a
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait implemented by handlers which can provided a boxed future
/// to a response type which can be turned into a response
///
/// `State`  The type of state provided to the handler
/// `Format` The format of the handler function (FormatA, FormatB)
/// `Req`    The request value type for the handler
/// `Res`    The response type for the handler
pub trait Handler<'a, State, Req, Res>: Send + Sync + 'static {
    /// Handle function for calling the underlying handle logic using
    /// the proivded state and packet
    ///
    /// `state`  The state to provide
    /// `packet` The packet to handle
    fn handle(&self, state: &'a mut State, req: Req) -> BoxFuture<'a, Res>;
}

/// Future which results in a response packet being produced that can
/// only live for the lifetime of 'a which is the state lifetime
type PacketFuture<'a> = BoxFuture<'a, Packet>;

/// Handler implementation for async functions that take the state as well
/// as a request type
///
/// ```
/// struct State;
/// struct Req;
/// struct Res;
///
/// async fn test(state: &mut State, req: Req) -> Res {
///     Res {}
/// }
/// ```
impl<'a, State, Fun, Fut, Req, Res> Handler<'a, State, Req, Res> for Fun
where
    Fun: Fn(&'a mut State, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'a,
    Req: FromRequest,
    Res: IntoResponse,
    State: Send + 'static,
{
    fn handle(&self, state: &'a mut State, req: Req) -> BoxFuture<'a, Res> {
        Box::pin(self(state, req))
    }
}

/// Handler implementation for async functions that take the state with no
/// request type
///
/// ```
/// struct State;
/// struct Res;
///
/// async fn test(state: &mut State) -> Res {
///     Res {}
/// }
/// ```
impl<'a, State, Fun, Fut, Res> Handler<'a, State, (), Res> for Fun
where
    Fun: Fn(&'a mut State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'a,
    Res: IntoResponse,
    State: Send + 'static,
{
    fn handle(&self, state: &'a mut State, _: ()) -> BoxFuture<'a, Res> {
        Box::pin(self(state))
    }
}

/// Future wrapper that wraps a future from a handler in order
/// to poll the underlying future and then transform the future
/// result into the response packet
///
/// 'a:   The lifetime of the session
/// `Res` The response type for the handler
struct HandlerFuture<'a, Res> {
    /// The future from the hanlder
    fut: BoxFuture<'a, Res>,
    /// The packet the handler is responding to
    packet: Packet,
}

impl<'a, Res> Future for HandlerFuture<'a, Res>
where
    Res: IntoResponse,
{
    type Output = Packet;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // Poll the underlying future
        let fut = Pin::new(&mut this.fut);
        let res = ready!(fut.poll(cx));
        // Transform the result
        let packet = res.into_response(&this.packet);
        Poll::Ready(packet)
    }
}

/// Trait for erasing the inner types of the handler routes
trait Route<S>: Send + Sync {
    /// Handle function for calling the handler logic on the actual implementation
    /// producing a future that lives as long as the state
    ///
    /// `state`  The state provided
    /// `packet` The packet to handle with the route
    fn handle<'s>(&self, state: &'s mut S, packet: Packet)
        -> Result<PacketFuture<'s>, HandleError>;
}

/// Route wrapper over a handler for storing the phantom type data
/// and implementing Route
struct HandlerRoute<H, Req, Res> {
    /// The underlying handler
    handler: H,
    /// Marker for storing related data
    _marker: PhantomData<fn(Req) -> Res>,
}

/// Route implementation for handlers wrapped by handler routes
impl<H, State, Req, Res> Route<State> for HandlerRoute<H, Req, Res>
where
    for<'a> H: Handler<'a, State, Req, Res>,
    Req: FromRequestInternal,
    Res: IntoResponse,
    State: Send + 'static,
{
    fn handle<'s>(
        &self,
        state: &'s mut State,
        packet: Packet,
    ) -> Result<PacketFuture<'s>, HandleError> {
        let req = match Req::from_request(&packet) {
            Ok(value) => value,
            Err(err) => return Err(HandleError::Decoding(err)),
        };
        let fut = self.handler.handle(state, req);
        Ok(Box::pin(HandlerFuture { fut, packet }))
    }
}

/// Route implementation for storing components mapped to route
/// handlers
pub struct Router<S> {
    /// The map of components to routes
    routes: HashMap<(u16, u16), Box<dyn Route<S>>>,
}

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self {
            routes: Default::default(),
        }
    }
}

impl<S> Router<S>
where
    S: Send + 'static,
{
    /// Creates a new router
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new route to the router where the route is something that implements
    /// the handler type with any lifetime. The value is wrapped with a HandlerRoute
    /// and stored boxed in the routes map under the component key
    ///
    /// `component` The component key for the route
    /// `route`     The actual route handler function
    pub fn route<Req, Res>(
        &mut self,
        component: (u16, u16),
        route: impl for<'a> Handler<'a, S, Req, Res>,
    ) where
        Req: FromRequestInternal,
        Res: IntoResponse,
    {
        self.routes.insert(
            component,
            Box::new(HandlerRoute {
                handler: route,
                _marker: PhantomData,
            }),
        );
    }

    /// Handle function takes the provided packet retrieves the component from its header
    /// and finds the matching route (Returning an empty response immediately if none match)
    /// and providing the state the route along with the packet awaiting the route future
    ///
    /// `state`  The provided state
    /// `packet` The packet to handle
    pub fn handle<'a>(
        &self,
        state: &'a mut S,
        packet: Packet,
    ) -> Result<PacketFuture<'a>, HandleError> {
        let target = (packet.header.component, packet.header.command);

        let route = match self.routes.get(&target) {
            Some(value) => value,
            None => return Err(HandleError::MissingHandler(packet)),
        };

        route.handle(state, packet)
    }
}

/// Error that can occur while handling a packet
#[derive(Debug)]
pub enum HandleError {
    /// There wasn't an available handler for the provided packet
    MissingHandler(Packet),
    /// Decoding error while reading the packet
    Decoding(DecodeError),
}
