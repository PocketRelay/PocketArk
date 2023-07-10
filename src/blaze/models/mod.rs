use std::time::{SystemTime, UNIX_EPOCH};

use blaze_pk::{
    codec::{Decodable, Encodable},
    error::DecodeResult,
    reader::TdfReader,
    tag::TdfType,
    writer::TdfWriter,
};
use log::debug;

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
    pub target: UpgradedTarget,
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

        debug!("{}", host_alt);

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

pub struct AuthNotify;

impl Encodable for AuthNotify {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_zero(b"\x17CON");
        w.tag_u32(b"ALOC", 1701727834);
        w.tag_u32(b"BUID", 978651371);
        w.tag_triple(b"CGID", (30722u64, 2u64, 1052287650009u64));
        w.tag_str(b"DSNM", "Jacobtread");
        w.tag_zero(b"FRST");
        w.tag_str(b"KEY", "0");
        w.tag_u32(b"LAST", 1688871852);
        w.tag_u32(b"LLOG", 1688871991);
        w.tag_str(b"MAIL", "******@gmail.com");
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_u32(b"PID", 978651371);
        w.tag_u8(b"PLAT", 4);
        w.tag_u64(b"UID", 1000279946559);
        w.tag_u8(b"USTP", 0);
        w.tag_u64(b"XREF", 1000279946559);
    }
}

pub struct AuthResponse;

impl Encodable for AuthResponse {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"SESS", |w| {
            w.tag_zero(b"\x17CON");
            w.tag_u32(b"BUID", 978651371);
            w.tag_zero(b"FRST");
            w.tag_str(b"KEY", "0");
            w.tag_u32(b"LLOG", 1688871991);
            w.tag_str(b"MAIL", "******@gmail.com");
            w.group(b"PDTL", |w| {
                w.tag_str(b"DSNM", "Jacobtread");
                w.tag_u32(b"LAST", 0);
                w.tag_u32(b"PID", 978651371);
                w.tag_u8(b"PLAT", 4);
                w.tag_u8(b"STAS", 0);
                w.tag_u64(b"XREF", 1000279946559);
            });
            w.tag_u64(b"UID", 1000279946559);
        });
        w.tag_u8(b"SPAM", 0);
        w.tag_u8(b"UNDR", 0);
    }
}

pub struct ClientConfigReq {
    pub cfid: String,
}

impl Decodable for ClientConfigReq {
    fn decode(reader: &mut TdfReader) -> DecodeResult<Self> {
        let cfid = reader.tag(b"CFID")?;
        Ok(ClientConfigReq { cfid })
    }
}

pub struct ClientConfigRes;
impl Encodable for ClientConfigRes {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_map_tuples(
            b"CONF",
            &[
                ("display", "console2/welcome"),
                ("redirect_uri", "http://127.0.0.1/success"),
            ],
        )
    }
}

pub struct PostAuthRes;

impl Encodable for PostAuthRes {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"TELE", |w| {
            w.tag_str(b"ADRS", "https://river.data.ea.com");
            w.tag_zero(b"ANON");
            w.tag_str(b"DISA", "AD,AF,AG,AI,AL,AM,AN,AO,AQ,AR,AS,AW,AX,AZ,BA,BB,BD,BF,BH,BI,BJ,BM,BN,BO,BR,BS,BT,BV,BW,BY,BZ,CC,CD,CF,CG,CI,CK,CL,CM,CN,CO,CR,CU,CV,CX,DJ,DM,DO,DZ,EC,EG,EH,ER,ET,FJ,FK,FM,FO,GA,GD,GE,GF,GG,GH,GI,GL,GM,GN,GP,GQ,GS,GT,GU,GW,GY,HM,HN,HT,ID,IL,IM,IN,IO,IQ,IR,IS,JE,JM,JO,KE,KG,KH,KI,KM,KN,KP,KR,KW,KY,KZ,LA,LB,LC,LI,LK,LR,LS,LY,MA,MC,MD,ME,MG,MH,ML,MM,MN,MO,MP,MQ,MR,MS,MU,MV,MW,MY,MZ,NA,NC,NE,NF,NG,NI,NP,NR,NU,OM,PA,PE,PF,PG,PH,PK,PM,PN,PS,PW,PY,QA,RE,RS,RW,SA,SB,SC,SD,SG,SH,SJ,SL,SM,SN,SO,SR,ST,SV,SY,SZ,TC,TD,TF,TG,TH,TJ,TK,TL,TM,TN,TO,TT,TV,TZ,UA,UG,UM,UY,UZ,VA,VC,VE,VG,VN,VU,WF,WS,YE,YT,ZM,ZW,ZZ");
            w.tag_zero(b"EDCT");
            w.tag_str(b"FILT", "-UION/****");
            w.tag_u32(b"LOC", 1701727834);
            w.tag_zero(b"MINR");
            w.tag_str(b"NOOK", "US,CA,MX");
            w.tag_u16(b"LOC", 443);
            w.tag_u16(b"SDLY", 15000);
            w.tag_str(b"SESS", "4QiqktOCVpD");
            w.tag_str(b"SKEY", "^�¦��Δ�ۍ��ڍ���騊�웱�䕋ƌ������������֦̉���ʉ��ؗ��͛�̙�����¦����ı�������ɣ�˲��Ҁ�");
            w.tag_u16(b"SPCT", 75);
            w.tag_str(b"STIM", "Default");
            w.tag_str(b"SVNM", "telemetry-3-common");
        });
        w.group(b"TICK", |w| {
            w.tag_str(b"ADRS", "10.23.15.2");
            w.tag_u16(b"PORT", 8999);
            w.tag_str(
                b"SKEY",
                "978651371,10.23.15.2:8999,masseffect-4-pc,10,50,50,50,50,0,12",
            );
        });
        w.group(b"UROP", |w| {
            w.tag_u8(b"TMOP", 1);
            w.tag_u32(b"UID", 978651371)
        })
    }
}

pub struct UpdateNetworkInfo {}

#[test]
fn test() {
    let bytes = [1, 130, 252, 237, 244, 20, 1, 1];
    let a = u64::from_be_bytes(bytes);
    println!("{}", a)
}
