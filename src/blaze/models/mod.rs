use std::time::{SystemTime, UNIX_EPOCH};

use blaze_pk::{
    codec::{Decodable, Encodable},
    error::DecodeResult,
    reader::TdfReader,
    tag::TdfType,
    writer::TdfWriter,
};

use crate::http::middleware::upgrade::{BlazeScheme, UpgradedTarget};

pub mod util {
    pub static COMPONENT: u16 = 9;
    pub static PRE_AUTH: u16 = 7;
}

pub mod user_sessions {
    pub static COMPONENT: u16 = 30722;
    pub static UPDATE_NETWORK_INFO: u16 = 20;
}

pub struct PreAuthResponse {
    target: UpgradedTarget,
}

impl Encodable for PreAuthResponse {
    fn encode(&self, w: &mut TdfWriter) {
        let host = &self.target.host;
        let port = &self.target.port.to_string();
        let secure = &matches!(self.target.scheme, BlazeScheme::Https).to_string();

        let host_alt = format!(
            "{}{}:{}",
            self.target.scheme.value(),
            self.target.host,
            self.target.port
        );

        w.tag_str(b"ASRC", "310335");
        w.tag_slice_list(
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
                    ("arubaHostname", &host_alt),
                    ("associationListSkipInitialSet", "1"),
                    ("autoReconnectEnabled", "0"),
                    // TODO: Replace bytevault with the local name
                    ("bytevaultHostname", host),
                    ("bytevaultPort", port),
                    ("bytevaultSecure", secure),
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
                    // TODO: Replace with local telemtry server
                    ("riverEnv", "prod"),
                    ("riverHost", &host_alt),
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

            {
                w.tag_map_start(b"LTPS", TdfType::String, TdfType::Group, 1);
                w.write_str("bio-dub");
                // TODO: Replace this host and port with the local QOS server when complete
                w.tag_str(b"PSA", "qos-prod-bio-dub-common-common.gos.ea.com");
                w.tag_u16(b"PSP", 17504);
                w.tag_str(b"SNA", "prod-sjc");
                w.tag_group_end();
            }

            w.tag_u32(b"TIME", 5000000);
        });

        w.tag_str(b"RSRC", "310335");
        w.tag_str(b"SVER", "Blaze 15.1.1.4.5 (CL# 1764921)\n");
    }
}

pub struct PingResponse {
    pub time: u64,
}

impl Encodable for PingResponse {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_u64(b"STIM", self.time);
    }
}

pub struct AuthRequest {
    pub token: String,
}

impl Decodable for AuthRequest {
    fn decode(reader: &mut TdfReader) -> DecodeResult<Self> {
        let token = reader.tag(b"AUTH")?;
        Ok(Self { token })
    }
}

pub struct UpdateNetworkInfo {}

#[test]
fn test() {
    let bytes = [1, 130, 252, 237, 244, 20, 1, 1];
    let a = u64::from_be_bytes(bytes);
    println!("{}", a)
}
