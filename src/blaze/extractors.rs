//! Router implementation for routing packet components to different functions
//! and automatically decoding the packet contents to the function type

use futures::future::BoxFuture;
use std::{future::ready, sync::Arc};

use crate::services::game::player::GamePlayer;
use pocket_ark_database::dto::users::UserDto;

use super::session::SessionLink;
use pocket_ark_blaze_networking::{
    error::BlazeError,
    router::{FromPacketRequest, PacketRequest},
};

/// Extracts the session authenticated player if one is present,
/// responds with [GlobalError::AuthenticationRequired] if there is none
pub struct SessionAuth(pub Arc<UserDto>);

impl FromPacketRequest<SessionLink> for SessionAuth {
    type Rejection = BlazeError;

    fn from_packet_request<'a>(
        req: &'a mut PacketRequest<SessionLink>,
    ) -> BoxFuture<'a, Result<Self, Self::Rejection>>
    where
        Self: 'a,
    {
        let user = req.state.data.get_player().expect("missing user");
        Box::pin(ready(Ok(SessionAuth(user))))
    }
}

impl FromPacketRequest<SessionLink> for GamePlayer {
    type Rejection = BlazeError;

    fn from_packet_request<'a>(
        req: &'a mut PacketRequest<SessionLink>,
    ) -> BoxFuture<'a, Result<Self, Self::Rejection>>
    where
        Self: 'a,
    {
        let user = req.state.data.get_player().expect("missing user");
        Box::pin(ready(Ok(GamePlayer::new(user, Arc::downgrade(&req.state)))))
    }
}
