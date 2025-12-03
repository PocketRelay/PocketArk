use crate::{
    blaze::{
        components::{self, game_manager::GAME_TYPE, user_sessions::PLAYER_SESSION_TYPE},
        data::NetData,
    },
    database::entity::{User, users::UserId},
};
use bitflags::bitflags;
use serde::Serialize;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
use tdf::prelude::*;

use super::util::PING_SITE_ALIAS;

#[derive(Debug, Clone, Copy, Default, Serialize, TdfSerialize, TdfDeserialize, TdfTyped)]
#[tdf(group)]
pub struct QosNetworkData {
    #[tdf(tag = "BWHR")]
    pub bwhr: u32,
    #[tdf(tag = "DBPS")]
    pub dbps: u32,
    #[tdf(tag = "NAHR")]
    pub nahr: u32,
    #[tdf(tag = "NATT")]
    pub natt: NatType,
    #[tdf(tag = "UBPS")]
    pub ubps: u32,
}

//
#[derive(Debug, Default, Copy, Clone, Serialize, TdfDeserialize, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum NatType {
    /// Players behind an open NAT can usually connect to any other player and are ideal game hosts.
    Open = 0x0,
    /// Players behind a moderate NAT can usually connect to other open or moderate players.
    Moderate = 0x1,
    /// Players behind a strict (but sequential) NAT can usually only connect to open players and are poor game hosts.
    StrictSequential = 0x2,
    /// Players behind a strict (non-sequential) NAT can usually only connect to open players and are the worst game hosts.
    Strict = 0x3,
    /// unknown NAT type; possibly timed out trying to detect NAT.
    #[default]
    #[tdf(default)]
    Unknown = 0x4,
}

#[derive(Default, Debug, Clone, TdfSerialize, TdfDeserialize, TdfTyped, Serialize)]
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
#[derive(Debug, Clone, Serialize, TdfDeserialize, TdfSerialize, TdfTyped)]
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

#[derive(Debug, Clone, Serialize, TdfDeserialize, TdfSerialize, TdfTyped)]
#[tdf(group)]
pub struct PairAddress {
    #[tdf(tag = "IP", into = u32)]
    pub addr: Ipv4Addr,
    #[tdf(tag = "MACI")]
    pub maci: u32,
    #[tdf(tag = "PORT")]
    pub port: u16,
}

/// Request to update the stored networking information for a session
#[derive(Debug, TdfDeserialize)]
pub struct UpdateNetworkRequest {
    #[tdf(tag = "INFO")]
    pub info: NetworkInfo,
}

#[derive(Debug)]
pub struct NetworkInfo {
    /// The client address net groups
    pub address: NetworkAddress,
    /// Latency to the different ping sites
    pub ping_site_latency: Option<TdfMap<String, u32>>,
    /// The client Quality of Service data
    pub qos: QosNetworkData,
}

// Contains optional field so must manually deserialize
impl TdfDeserializeOwned for NetworkInfo {
    fn deserialize_owned(
        r: &mut tdf::prelude::TdfDeserializer<'_>,
    ) -> tdf::prelude::DecodeResult<Self> {
        let address: NetworkAddress = r.tag(b"ADDR")?;
        let ping_site_latency: Option<TdfMap<String, u32>> = r.try_tag(b"NLMP")?;
        let qos: QosNetworkData = r.tag(b"NQOS")?;
        tdf::GroupSlice::deserialize_content_skip(r)?;

        Ok(Self {
            address,
            ping_site_latency,
            qos,
        })
    }
}

impl TdfTyped for NetworkInfo {
    const TYPE: TdfType = TdfType::Group;
}

#[derive(Debug, TdfDeserialize)]
pub struct UpdateHardwareFlags {
    /// The hardware flag value
    #[tdf(tag = "HWFG", into = u8)]
    pub hardware_flags: HardwareFlags,
}

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
    pub struct HardwareFlags: u8 {
        const NONE = 0;
        const VOIP_HEADSET_STATUS = 1;
    }
}

impl From<HardwareFlags> for u8 {
    #[inline]
    fn from(value: HardwareFlags) -> Self {
        value.bits()
    }
}

impl From<u8> for HardwareFlags {
    #[inline]
    fn from(value: u8) -> Self {
        HardwareFlags::from_bits_retain(value)
    }
}

#[derive(TdfSerialize)]
pub struct UserSessionExtendedDataUpdate {
    #[tdf(tag = "DATA")]
    pub data: UserSessionExtendedData,
    // Total number of subscribers?
    #[tdf(tag = "SUBS")]
    pub subs: usize,
    // The user ID that the session data is for
    #[tdf(tag = "USID")]
    pub user_id: UserId,
}

#[derive(TdfTyped)]
#[tdf(group)]
pub struct UserSessionExtendedData {
    /// Networking data for the session
    pub net: Arc<NetData>,
    /// ID of the game the player is in (if present)
    pub game: Option<u32>,

    pub user_id: UserId,
}

impl TdfSerialize for UserSessionExtendedData {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group_body(|w| {
            // Network address
            w.tag_ref(b"ADDR", &self.net.addr);
            // Best ping site alias
            w.tag_str(b"BPS", PING_SITE_ALIAS);
            // Country
            w.tag_str(b"CTY", "NZ");
            // Client data
            w.tag_var_int_list_empty(b"CVAR");
            // Data map
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
            // Hardware flags
            w.tag_owned(b"HWFG", self.net.hardware_flags.bits());
            // Internet Service Provider
            w.tag_str(b"ISP", "Example ISP");
            // Ping server latency list
            w.tag_list_slice(b"PSLM", &[0xfff0fff]);

            // Quality of service data
            w.tag_ref(b"QDAT", &self.net.qos);
            // Timezone
            w.tag_str(b"TZ", "Pacific/Auckland");

            // User info attributes
            w.tag_owned(b"UATT", 0u8);

            let session_id = ObjectId::new(PLAYER_SESSION_TYPE, self.user_id as u64);

            if let Some(game) = self.game {
                // Blaze object ID list
                w.tag_list_slice(
                    b"ULST",
                    &[session_id, ObjectId::new(GAME_TYPE, game as u64)],
                );
            } else {
                // Blaze object ID list
                w.tag_list_slice(b"ULST", &[session_id]);
            }
        });
    }
}

#[derive(TdfTyped)]
#[tdf(group)]
pub struct UserIdentification<'a> {
    pub id: u32,
    pub name: &'a str,
}

impl<'a> UserIdentification<'a> {
    pub fn from_user(user: &'a User) -> Self {
        Self {
            id: user.id,
            name: &user.username,
        }
    }
}

impl TdfSerialize for UserIdentification<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group_body(|w| {
            // Account ID
            w.tag_owned(b"AID", self.id);
            // Account locale
            w.tag_owned(b"ALOC", 0x64654445u32);
            // External blob
            w.tag_blob_empty(b"EXBB");
            // External ID
            w.tag_owned(b"EXID", self.id);
            // Blaze ID
            w.tag_owned(b"ID", self.id);
            // Account name
            w.tag_str(b"NAME", self.name);
            // Namespace?
            w.tag_str(b"NASP", "cem_ea_id");
            w.tag_owned(b"ORIG", self.id);
            w.tag_owned(b"PIDI", self.id);
        });
    }
}

#[derive(TdfSerialize)]
pub struct NotifyUserAdded<'a> {
    /// The user session data
    #[tdf(tag = "DATA")]
    pub session_data: UserSessionExtendedData,
    /// The added user identification
    #[tdf(tag = "USER")]
    pub user: UserIdentification<'a>,
}

#[derive(TdfSerialize)]
pub struct NotifyUserRemoved {
    /// The ID of the removed user
    #[tdf(tag = "BUID")]
    pub user_id: UserId,
}

#[derive(TdfSerialize)]
pub struct NotifyUserUpdated {
    #[tdf(tag = "FLGS", into = u8)]
    pub flags: UserDataFlags,
    /// The ID of the updated user
    #[tdf(tag = "ID")]
    pub user_id: UserId,
}

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
    pub struct UserDataFlags: u8 {
        const NONE = 0;
        const SUBSCRIBED = 1;
        const ONLINE = 2;
    }
}

impl From<UserDataFlags> for u8 {
    fn from(value: UserDataFlags) -> Self {
        value.bits()
    }
}

impl From<u8> for UserDataFlags {
    fn from(value: u8) -> Self {
        UserDataFlags::from_bits_retain(value)
    }
}
