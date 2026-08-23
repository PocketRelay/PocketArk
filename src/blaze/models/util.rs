use std::{borrow::Cow, net::Ipv4Addr};

use crate::{
    blaze::models::constants::{
        BLAZE_VERSION, BYTEVAULT_HOSTNAME, LOCALE_NZ, REGISTRATION_SOURCE, RIVER_HOST,
        TELEMETRY_ADDRS, TELEMETRY_DISA, TELEMETRY_KEY, TICKER_KEY,
    },
    database::entity::users::UserId,
    utils::constants::LOCAL_HTTP_PORT,
};
use bitflags::bitflags;
use tdf::prelude::*;

pub struct PreAuthResponse;

impl TdfSerialize for PreAuthResponse {
    fn serialize<S: TdfSerializer>(&self, w: &mut S) {
        let port = &LOCAL_HTTP_PORT.to_string();
        let host_target = format!("https://localhost:{}/", LOCAL_HTTP_PORT);

        w.tag_str(b"ASRC", "310335");
        w.tag_list_slice(
            b"CIDS",
            &[
                1, 4, 7, 9, 10, 11, 14, 15, 25, 2000, 27, 30720, 30721, 30722, 30723, 30724, 33,
                30725, 30726, 30727, 30728, 30729, 30730, 63490,
            ],
        );
        w.tag_str(b"CLID", "ME4-PC-SERVER-BLAZE");
        w.group(b"CONF", |w| {
            w.tag_map_tuples(
                b"CONF",
                &[
                    ("arubaDisabled", "false"),
                    ("arubaEndpoint", "PROD"),
                    ("arubaHostname", &host_target),
                    ("associationListSkipInitialSet", "1"),
                    ("autoReconnectEnabled", "0"),
                    ("bytevaultHostname", BYTEVAULT_HOSTNAME),
                    ("bytevaultPort", port),
                    ("bytevaultSecure", "true"),
                    ("cachedUserRefreshInterval", "1s"),
                    ("connIdleTimeout", "40s"),
                    ("defaultRequestTimeout", "20s"),
                    ("disableDisconnectOnOrbitError", "false"),
                    ("maxReconnectAttempts", "30"),
                    ("nucleusConnect", "https://accounts.ea.com"),
                    ("nucleusConnectTrusted", "https://accounts2s.ea.com"),
                    ("nucleusPortal", "https://signin.ea.com"),
                    ("nucleusProxy", "https://gateway.ea.com"),
                    ("pingPeriod", "20s"),
                    ("riverEnv", "prod"),
                    ("riverHost", RIVER_HOST),
                    ("riverPort", port),
                    ("userManagerMaxCachedUsers", "0"),
                    ("voipHeadsetUpdateRate", "1000"),
                    ("xblTokenUrn", "accounts.ea.com"),
                    ("xboxOneStringValidationUri", "client-strings.xboxlive.com"),
                ],
            );
        });
        w.tag_str(b"ESRC", "310335");
        w.tag_str(b"INST", "masseffect-4-pc");
        w.tag_u32(b"MAID", 2291763061);
        w.tag_zero(b"MINR");
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_str_empty(b"PILD");
        w.tag_str(b"PLAT", "pc");

        w.group(b"QOSS", |w| {
            w.group(b"BWPS", |w| {
                w.tag_str_empty(b"PSA");
                w.tag_zero(b"PSP");
            });

            w.tag_u8(b"LNP", 10);

            let official_ping_sites = official_ping_sites();
            w.tag_map_tuples(b"LTPS", &official_ping_sites);
            // w.tag_map_tuples(
            //     b"LTPS",
            //     &[(
            //         "bio-dub".to_string(),
            //         PingSiteAlias {
            //             alias: "localhost".to_string(),
            //             port: LOCAL_HTTP_PORT,
            //         },
            //     )],
            // );

            w.tag_u32(b"TIME", 5000000);
        });

        w.tag_str(b"RSRC", REGISTRATION_SOURCE);
        w.tag_str(b"SVER", BLAZE_VERSION);
    }
}

#[derive(TdfSerialize, TdfTyped)]
#[tdf(group)]
pub struct PingSiteAlias {
    #[tdf(tag = "PSA")]
    alias: String,
    #[tdf(tag = "PSP")]
    port: u16,
}

fn official_ping_sites() -> [(String, PingSiteAlias); 6] {
    [
        (
            "bio-dub".to_string(),
            PingSiteAlias {
                alias: "qos-prod-bio-dub-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
        (
            "bio-iad".to_string(),
            PingSiteAlias {
                alias: "qos-prod-bio-iad-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
        (
            "bio-sjc".to_string(),
            PingSiteAlias {
                alias: "qos-prod-bio-sjc-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
        (
            "bio-syd".to_string(),
            PingSiteAlias {
                alias: "qos-prod-bio-syd-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
        (
            "m3d-brz".to_string(),
            PingSiteAlias {
                alias: "qos-prod-m3d-brz-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
        (
            "m3d-nrt".to_string(),
            PingSiteAlias {
                alias: "qos-prod-m3d-nrt-common-common.gos.ea.com".to_string(),
                port: 17504,
            },
        ),
    ]
}

#[derive(TdfSerialize)]
pub struct PingResponse {
    #[tdf(tag = "STIM")]
    pub time: u64,
}

pub struct PostAuthResponse {
    pub user_id: UserId,
}

impl TdfSerialize for PostAuthResponse {
    fn serialize<S: TdfSerializer>(&self, w: &mut S) {
        w.group(b"TELE", |w| {
            w.tag_str(b"ADRS", TELEMETRY_ADDRS);
            w.tag_zero(b"ANON");
            w.tag_str(b"DISA", TELEMETRY_DISA);
            w.tag_zero(b"EDCT");
            w.tag_str(b"FILT", "-UION/****");
            w.tag_u32(b"LOC", LOCALE_NZ);
            w.tag_zero(b"MINR");
            w.tag_str(b"NOOK", "US,CA,MX,NZ");
            w.tag_u16(b"PORT", LOCAL_HTTP_PORT);
            w.tag_u16(b"SDLY", 15000);
            w.tag_str(b"SESS", "4QiqktOCVpD");

            let key: Cow<str> = String::from_utf8_lossy(TELEMETRY_KEY);

            w.tag_str(b"SKEY", &key);
            w.tag_u16(b"SPCT", 75);
            w.tag_str(b"STIM", "Default");
            w.tag_str(b"SVNM", "telemetry-3-common");
        });
        w.group(b"TICK", |w| {
            // TODO: Replace tick server
            w.tag_str(b"ADRS", "10.23.15.2");
            w.tag_u16(b"PORT", 8999);
            w.tag_str(b"SKEY", TICKER_KEY);
        });
        w.group(b"UROP", |w| {
            w.tag_u8(b"TMOP", 1);
            w.tag_u32(b"UID", self.user_id)
        });
    }
}

#[derive(Debug, TdfDeserialize)]
pub struct ClientConfigRequest {
    #[tdf(tag = "CFID")]
    pub id: String,
}

#[derive(Debug, TdfSerialize)]
pub struct ClientConfigResponse {
    #[tdf(tag = "CONF")]
    pub config: TdfMap<&'static str, &'static str>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct UpnpFlags: u16 {
        /// NAT type promoted from Moderate to Open due to UPnP success result.
        const NAT_PROMOTED = 0x1;
        /// WAN IP address does not match IP address seen by Blaze server.
        const DOUBLE_NAT = 0x2;
        /// External port derived by QoS was overridden by UPnP external port.
        const PORT_OVERRIDE = 0x4;
    }
}

impl From<u16> for UpnpFlags {
    fn from(value: u16) -> Self {
        Self::from_bits_retain(value)
    }
}

#[derive(Default, Debug, Clone, Copy, TdfDeserialize, TdfTyped)]
#[repr(u8)]
pub enum UpnpStatus {
    /// Upnp status unknown.
    #[default]
    #[tdf(default)]
    Unknown = 0,
    /// Upnp found, but not fully working.
    Found = 1,
    /// Upnp is enabled (found and port mapping added).
    Enabled = 2,
}

/// Contains UPnP data such as status flags, device info, etc.
// #[derive(TdfDeserialize)]
pub struct SetClientMetricsRequest {
    /// pnp Blaze status flags.
    // #[tdf(tag = "UBFL", into = u16)]
    pub blaze_flags: UpnpFlags,

    /// Upnp device info.
    // #[tdf(tag = "UDEV")]
    pub device_info: String,

    /// Upnp status flags.
    // #[tdf(tag = "UFLG")]
    pub flags: u16,

    /// Upnp last result code.
    // #[tdf(tag = "ULRC")]
    #[allow(unused)]
    pub last_result_code: i32,

    /// Upnp metrics report NAT type.
    // #[tdf(tag = "UNAT")]
    pub nat_type: u16,

    /// Upnp status.
    // #[tdf(tag = "USTA")]
    pub status: UpnpStatus,

    /// WAN IP address
    // #[tdf(tag = "UWAN", into = u32)]
    pub wan: Option<Ipv4Addr>,
}

impl tdf::TdfDeserialize<'_> for SetClientMetricsRequest {
    fn deserialize(r: &mut tdf::TdfDeserializer<'_>) -> tdf::DecodeResult<Self> {
        let blaze_flags = <UpnpFlags as From<u16>>::from(r.tag::<u16>(&[85u8, 66u8, 70u8, 76u8])?);
        let device_info = r.tag::<String>(&[85u8, 68u8, 69u8, 86u8])?;
        let flags = r.tag::<u16>(&[85u8, 70u8, 76u8, 71u8])?;
        let last_result_code = r.tag::<i32>(&[85u8, 76u8, 82u8, 67u8])?;
        let nat_type = r.tag::<u16>(&[85u8, 78u8, 65u8, 84u8])?;
        let status = r.tag::<UpnpStatus>(&[85u8, 83u8, 84u8, 65u8])?;
        let wan = r
            .try_tag::<u32>(&[85u8, 87u8, 65u8, 78u8])?
            .map(Ipv4Addr::from);
        Ok(Self {
            blaze_flags,
            device_info,
            flags,
            last_result_code,
            nat_type,
            status,
            wan,
        })
    }
}
