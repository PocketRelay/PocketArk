use std::sync::Arc;

use tdf::ObjectId;

use crate::{
    blaze::{
        components::user_sessions::PLAYER_SESSION_TYPE,
        data::NetData,
        models::{game_manager::PlayerState, user_sessions::NetworkAddress},
        packet::Packet,
        session::WeakSessionLink,
    },
    database::entity::{User, users::UserId},
    services::game::AttrMap,
};

pub struct GamePlayer {
    pub user: Arc<User>,
    pub link: WeakSessionLink,
    pub net: Arc<NetData>,
    pub state: PlayerState,
    pub attr: AttrMap,
}

impl Drop for GamePlayer {
    fn drop(&mut self) {
        self.try_clear_game();
    }
}

impl GamePlayer {
    pub fn new(user: Arc<User>, link: WeakSessionLink, net: Arc<NetData>) -> Self {
        Self {
            user,
            link,
            net,
            state: PlayerState::ActiveConnecting,
            attr: AttrMap::default(),
        }
    }

    #[allow(unused)]
    pub fn net(&self) -> Option<Arc<NetData>> {
        let session = self.link.upgrade()?;
        session.data.net()
    }

    #[allow(unused)]
    pub fn network_address(&self) -> NetworkAddress {
        match self.net() {
            Some(net) => net.addr.clone(),
            None => NetworkAddress::Unset,
        }
    }

    pub fn try_clear_game(&self) {
        if let Some(link) = self.link.upgrade() {
            link.data.clear_game_gm();
        }
    }

    pub fn try_subscribe(&self, player_id: UserId, subscriber: WeakSessionLink) {
        if let Some(link) = self.link.upgrade() {
            link.data.add_subscriber(player_id, subscriber);
        }
    }

    pub fn try_unsubscribe(&self, player_id: UserId) {
        if let Some(link) = self.link.upgrade() {
            link.data.remove_subscriber(player_id);
        }
    }

    #[inline]
    pub fn notify(&self, packet: Packet) {
        let session = match self.link.upgrade() {
            Some(value) => value,
            // Session is already closed
            None => return,
        };

        session.tx.notify(packet)
    }

    pub fn encode<S: tdf::TdfSerializer>(&self, game_id: u32, slot: usize, w: &mut S) {
        w.tag_blob_empty(b"BLOB");
        w.tag_owned(b"CONG", self.user.id);
        w.tag_u8(b"CSID", 0);
        w.tag_u8(b"DSUI", 0);
        w.tag_blob_empty(b"EXBL");
        w.tag_owned(b"EXID", self.user.id);
        w.tag_owned(b"GID", game_id);
        w.tag_u8(b"JFPS", 1);
        w.tag_u8(b"JVMM", 1);
        w.tag_u32(b"LOC", 0x64654445);
        w.tag_str(b"NAME", &self.user.username);
        w.tag_str(b"NASP", "cem_ea_id");
        if !self.attr.is_empty() {
            w.tag_ref(b"PATT", &self.attr);
        }
        w.tag_u32(b"PID", self.user.id);
        w.tag_ref(b"PNET", &self.net.addr);

        w.tag_u8(b"PSET", 1);
        w.tag_u8(b"RCRE", 0);
        w.tag_str_empty(b"ROLE");
        w.tag_usize(b"SID", slot);
        w.tag_u8(b"SLOT", 0);
        w.tag_ref(b"STAT", &self.state);
        w.tag_u16(b"TIDX", 0);
        w.tag_u8(b"TIME", 0); /* Unix timestamp in milliseconds */
        // User group ID
        w.tag_alt(
            b"UGID",
            ObjectId::new(PLAYER_SESSION_TYPE, self.user.id as u64),
        );

        w.tag_owned(b"UID", self.user.id);

        let uuid = self
            .link
            .upgrade()
            .map(|value| value.id.to_string())
            .unwrap_or_default();

        w.tag_str(b"UUID", &uuid);
        w.tag_group_end();
    }
}
