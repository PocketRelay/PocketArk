use std::time::{SystemTime, UNIX_EPOCH};

use blaze_pk::{codec::Encodable, tag::TdfType, writer::TdfWriter};

pub mod util {
    pub static COMPONENT: u16 = 9;
    pub static PRE_AUTH: u16 = 7;
}

pub struct PreAuthResponse;

impl Encodable for PreAuthResponse {
    fn encode(&self, w: &mut TdfWriter) {
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
                    ("arubaHostname", "https://pin-em.data.ea.com/"),
                    ("associationListSkipInitialSet", "1"),
                    ("autoReconnectEnabled", "0"),
                    // TODO: Replace bytevault with the local name
                    ("bytevaultHostname", "mea-public.biowareonline.net"),
                    ("bytevaultPort", "443"),
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
                    // TODO: Replace with local telemtry server
                    ("riverEnv", "prod"),
                    ("riverHost", "https://pin-river.data.ea.com"),
                    ("riverPort", "443"),
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
