use crate::blaze::components::{
    self, game_manager::GAME_INSTANCE_TYPE, user_sessions::PLAYER_SESSION_TYPE,
};
use std::net::{Ipv4Addr, SocketAddrV4};
use tdf::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct NetData {
    pub addr: NetworkAddress,
    pub qos: QosNetworkData,
    pub hwfg: u8,
}

pub struct UserUpdated {
    pub player_id: u32,
    pub game_id: Option<u32>,
    pub net_data: NetData,
}

impl TdfSerialize for UserUpdated {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group(b"DATA", |w| {
            // ADDR
            w.tag_ref(b"ADDR", &self.net_data.addr);
            w.tag_str(b"BPS", "bio-syd");
            w.tag_str(b"CTY", "NZ"); // Country
            w.tag_var_int_list_empty(b"CVAR");
            w.tag_map_tuples(
                b"DMAP",
                &[
                    (458788, 0),
                    (458789, 0),
                    (458790, 0),
                    (458791, 0),
                    (458792, 0),
                    (458877, 0),
                    (917505, 0),
                    (917506, 0),
                    (2013396993, 0),
                ],
            );
            w.tag_u8(b"HWFG", self.net_data.hwfg); // Hardware config
            w.tag_str(b"ISP", "Example ISP"); // Internet Service Provider
            w.tag_list_slice(b"PSLM", &[296, 245, 153, 40, 312, 238]);
            w.tag_ref(b"QDAT", &self.net_data.qos);
            w.tag_str(b"TZ", "Pacific/Auckland"); // Timezone
            w.tag_zero(b"UATT");

            w.tag_list_start(
                b"ULST",
                TdfType::ObjectId,
                if self.game_id.is_some() { 2 } else { 1 },
            );
            ObjectId::new(PLAYER_SESSION_TYPE, self.player_id as u64).serialize(w);
            if let Some(game_id) = &self.game_id {
                ObjectId::new(GAME_INSTANCE_TYPE, *game_id as u64).serialize(w)
            }
        });

        w.tag_u8(b"SUBS", 1);
        w.tag_owned(b"USID", self.player_id);
    }
}

pub struct UserAdded {
    pub name: String,
    pub player_id: u32,
    pub game_id: Option<u32>,
    pub net_data: NetData,
}

impl TdfSerialize for UserAdded {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group(b"DATA", |w| {
            // ADDR
            w.tag_ref(b"ADDR", &self.net_data.addr);
            w.tag_str_empty(b"BPS");
            w.tag_str(b"CTY", "NZ"); // Country
            w.tag_var_int_list_empty(b"CVAR");
            w.tag_map_tuples(
                b"DMAP",
                &[
                    (458788, 0),
                    (458789, 0),
                    (458790, 0),
                    (458791, 0),
                    (458792, 0),
                    (458877, 0),
                    (917505, 0),
                    (917506, 0),
                    (2013396993, 0),
                ],
            );
            w.tag_u8(b"HWFG", self.net_data.hwfg); // Hardware config
            w.tag_str(b"ISP", "Example ISP"); // Internet Service Provider
            w.tag_ref(b"QDAT", &self.net_data.qos);
            w.tag_str(b"TZ", "Pacific/Auckland"); // Timezone
            w.tag_zero(b"UATT");

            w.tag_list_start(
                b"ULST",
                TdfType::ObjectId,
                if self.game_id.is_some() { 2 } else { 1 },
            );

            ObjectId::new(PLAYER_SESSION_TYPE, self.player_id as u64).serialize(w);
            if let Some(game_id) = &self.game_id {
                ObjectId::new(GAME_INSTANCE_TYPE, *game_id as u64).serialize(w)
            }
        });

        w.group(b"USER", |w| {
            w.tag_owned(b"AID", self.player_id);
            w.tag_u32(b"ALOC", 1701727834);
            w.tag_blob_empty(b"EXBB");
            w.tag_owned(b"EXID", self.player_id);
            w.tag_owned(b"ID", self.player_id);
            w.tag_str(b"NAME", &self.name);
            w.tag_str(b"NASP", "cem_ea_id");
            w.tag_owned(b"ORIG", self.player_id);
            w.tag_owned(b"PIDI", self.player_id);
        });
    }
}

#[derive(Debug, Clone, Default, TdfSerialize, TdfDeserialize, TdfTyped)]
#[tdf(group)]
pub struct QosNetworkData {
    #[tdf(tag = "BWHR")]
    pub bwhr: u32,
    #[tdf(tag = "DBPS")]
    pub dbps: u32,
    #[tdf(tag = "NAHR")]
    pub nahr: u32,
    #[tdf(tag = "NATT")]
    pub natt: u8,
    #[tdf(tag = "UBPS")]
    pub ubps: u32,
}

#[derive(Default, Debug, Clone, TdfSerialize, TdfDeserialize, TdfTyped)]
pub enum NetworkAddress {
    #[tdf(key = 0x2, tag = "VALU")]
    AddressPair(IpPairAddress),
    #[tdf(unset)]
    Unset,
    #[default]
    #[tdf(default)]
    Default,
}

/// Pair of socket addresses
#[derive(Debug, Clone, TdfDeserialize, TdfSerialize, TdfTyped)]
#[tdf(group)]
pub struct IpPairAddress {
    #[tdf(tag = "EXIP")]
    pub external: PairAddress,
    #[tdf(tag = "INIP")]
    pub internal: PairAddress,
    #[tdf(tag = "MACI")]
    pub maci: u32,
}

impl IpPairAddress {
    pub fn tag<S: tdf::TdfSerializer>(addr: Option<&IpPairAddress>, tag: &[u8], w: &mut S) {
        if let Some(addr) = addr {
            w.tag_union_value(b"ADDR", 0x2, b"VALU", addr);
        } else {
            w.tag_union_unset(b"ADDR");
        }
    }
}

#[derive(Debug, Clone, TdfDeserialize, TdfSerialize, TdfTyped)]
#[tdf(group)]
pub struct PairAddress {
    #[tdf(tag = "IP", into = u32)]
    pub addr: Ipv4Addr,
    #[tdf(tag = "MACI")]
    pub maci: u32,
    #[tdf(tag = "PORT")]
    pub port: u16,
}

#[derive(Debug, TdfDeserialize)]
pub struct UpdateNetworkInfo {
    #[tdf(tag = "INFO")]
    pub info: NetworkInfo,
}

#[derive(Debug, TdfDeserialize, TdfTyped)]
#[tdf(group)]
pub struct NetworkInfo {
    #[tdf(tag = "ADDR")]
    pub addr: NetworkAddress,
    #[tdf(tag = "NQOS")]
    pub qos: QosNetworkData,
}

#[derive(Debug, TdfDeserialize)]
pub struct UpdateHardwareFlags {
    #[tdf(tag = "HWFG")]
    pub flags: u8,
}
