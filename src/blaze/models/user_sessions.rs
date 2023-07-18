use std::net::{Ipv4Addr, SocketAddrV4};

use serde_json::Value;

use crate::blaze::{
    components,
    pk::{
        codec::{Decodable, Encodable, ValueType},
        error::DecodeResult,
        reader::TdfReader,
        tag::TdfType,
        types::Union,
        writer::TdfWriter,
    },
};

#[derive(Default, Debug, Clone)]
pub struct NetData {
    pub addr: Option<IpPairAddress>,
    pub qos: QosNetworkData,
    pub hwfg: u8,
}

pub struct UserUpdated {
    pub player_id: u32,
    pub game_id: Option<u32>,
    pub net_data: NetData,
}

impl Encodable for UserUpdated {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"DATA", |w| {
            // ADDR
            IpPairAddress::tag(self.net_data.addr.as_ref(), b"ADDR", w);

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
            w.tag_slice_list(b"PSLM", &[296, 245, 153, 40, 312, 238]);
            w.tag_value(b"QDAT", &self.net_data.qos);
            w.tag_str(b"TZ", "Pacific/Auckland"); // Timezone
            w.tag_zero(b"UATT");

            w.tag_list_start(
                b"ULST",
                TdfType::Triple,
                if self.game_id.is_some() { 2 } else { 1 },
            );

            (components::user_sessions::COMPONENT, 2, self.player_id).encode(w);
            if let Some(game_id) = &self.game_id {
                (components::game_manager::COMPONENT, 1, *game_id).encode(w);
            }
        });

        w.tag_u8(b"SUBS", 1);
        w.tag_u32(b"USID", self.player_id);
    }
}

pub struct UserAdded {
    pub name: String,
    pub player_id: u32,
    pub game_id: Option<u32>,
    pub net_data: NetData,
}

impl Encodable for UserAdded {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"DATA", |w| {
            // ADDR
            IpPairAddress::tag(self.net_data.addr.as_ref(), b"ADDR", w);

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
            w.tag_value(b"QDAT", &self.net_data.qos);
            w.tag_str(b"TZ", "Pacific/Auckland"); // Timezone
            w.tag_zero(b"UATT");

            w.tag_list_start(
                b"ULST",
                TdfType::Triple,
                if self.game_id.is_some() { 2 } else { 1 },
            );

            (components::user_sessions::COMPONENT, 2, self.player_id).encode(w);
            if let Some(game_id) = &self.game_id {
                (components::game_manager::COMPONENT, 1, *game_id).encode(w);
            }
        });

        w.group(b"USER", |w| {
            w.tag_u32(b"AID", self.player_id);
            w.tag_u32(b"ALOC", 1701727834);
            w.tag_empty_blob(b"EXBB");
            w.tag_u32(b"EXID", self.player_id);
            w.tag_u32(b"ID", self.player_id);
            w.tag_str(b"NAME", &self.name);
            w.tag_str(b"NASP", "cem_ea_id");
            w.tag_u32(b"ORIG", self.player_id);
            w.tag_u32(b"PIDI", self.player_id);
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct QosNetworkData {
    pub bwhr: u32,
    pub dbps: u32,
    pub nahr: u32,
    pub natt: u8,
    pub ubps: u32,
}
impl Encodable for QosNetworkData {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_u32(b"BWHR", self.bwhr);
        w.tag_u32(b"DBPS", self.dbps);
        w.tag_u32(b"NAHR", self.nahr);
        w.tag_u8(b"NATT", self.natt);
        w.tag_u32(b"UBPS", self.ubps);
        w.tag_group_end();
    }
}

impl Decodable for QosNetworkData {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let bwhr: u32 = r.tag(b"BWHR")?;
        let dbps: u32 = r.tag(b"DBPS")?;
        let nahr: u32 = r.tag(b"NAHR")?;
        let natt: u8 = r.tag(b"NATT")?;
        let ubps: u32 = r.tag(b"UBPS")?;
        r.read_byte()?;
        Ok(Self {
            bwhr,
            dbps,
            nahr,
            natt,
            ubps,
        })
    }
}
impl ValueType for QosNetworkData {
    fn value_type() -> TdfType {
        TdfType::Group
    }
}

/// Pair of socket addresses
#[derive(Debug, Clone)]
pub struct IpPairAddress {
    pub external: PairAddress,
    pub internal: PairAddress,
    pub maci: u32,
}

impl IpPairAddress {
    pub fn tag(addr: Option<&IpPairAddress>, tag: &[u8], w: &mut TdfWriter) {
        if let Some(addr) = addr {
            w.tag_union_value(b"ADDR", 0x2, b"VALU", addr);
        } else {
            w.tag_union_unset(b"ADDR");
        }
    }
}

impl ValueType for IpPairAddress {
    fn value_type() -> TdfType {
        TdfType::Group
    }
}

impl Decodable for IpPairAddress {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let external = r.tag(b"EXIP")?;
        let internal = r.tag(b"INIP")?;
        let maci = r.tag(b"MACI")?;
        r.read_byte()?;
        Ok(Self {
            external,
            internal,
            maci,
        })
    }
}

impl Encodable for IpPairAddress {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_value(b"EXIP", &self.external);
        w.tag_value(b"INIP", &self.internal);
        w.tag_u32(b"MACI", self.maci);
        w.tag_group_end()
    }
}

#[derive(Debug, Clone)]
pub struct PairAddress {
    pub addr: SocketAddrV4,
    pub maci: u32,
}

impl Decodable for PairAddress {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let ip: u32 = r.tag(b"IP")?;
        let maci: u32 = r.tag(b"MACI")?;
        let port: u16 = r.tag(b"PORT")?;
        r.read_byte()?;

        let addr = SocketAddrV4::new(Ipv4Addr::from(ip), port);
        Ok(Self { addr, maci })
    }
}

impl Encodable for PairAddress {
    fn encode(&self, w: &mut TdfWriter) {
        let octets = self.addr.ip().octets();
        let value = u32::from_be_bytes(octets);

        w.tag_u32(b"IP", value);
        w.tag_u32(b"MACI", self.maci);
        w.tag_u16(b"PORT", self.addr.port());
        w.tag_group_end()
    }
}

impl ValueType for PairAddress {
    fn value_type() -> TdfType {
        TdfType::Group
    }
}

pub struct UpdateNetworkInfo {
    pub addr: Option<IpPairAddress>,
    pub qos: QosNetworkData,
}

impl Decodable for UpdateNetworkInfo {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        r.until_tag(b"INFO", TdfType::Group)?;
        let addr = match r.tag::<Union<IpPairAddress>>(b"ADDR")? {
            Union::Set { value, .. } => Some(value),
            Union::Unset => None,
        };
        let qos = r.tag(b"NQOS")?;
        Ok(Self { addr, qos })
    }
}

pub struct UpdateHardwareFlags {
    pub flags: u8,
}
impl Decodable for UpdateHardwareFlags {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let flag = r.tag(b"HWFG")?;
        Ok(Self { flags: flag })
    }
}
