use crate::utils::constants::LOCAL_HTTP_PORT;
use tdf::prelude::*;

/// Alias used for ping sites
pub const PING_SITE_ALIAS: &str = "bio-dub";

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
                    // TODO: Replace bytevault with the local name
                    ("bytevaultHostname", "localhost"),
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
                    // TODO: Replace with local telemtry server
                    ("riverEnv", "prod"),
                    ("riverHost", "https://localhost:42230"),
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
                w.group_body(|w| {
                    PING_SITE_ALIAS.serialize(w);
                    // TODO: Replace this host and port with the local QOS server when complete
                    w.tag_str(b"PSA", "localhost");
                    w.tag_u16(b"PSP", LOCAL_HTTP_PORT);
                    w.tag_str(b"SNA", "prod-sjc");
                });
            }

            w.tag_u32(b"TIME", 5000000);
        });

        w.tag_str(b"RSRC", "310335");
        w.tag_str(b"SVER", "Blaze 15.1.1.4.5 (CL# 1764921)\n");
    }
}

#[derive(TdfSerialize)]
pub struct PingResponse {
    #[tdf(tag = "STIM")]
    pub time: u64,
}

pub struct PostAuthResponse;

impl TdfSerialize for PostAuthResponse {
    fn serialize<S: TdfSerializer>(&self, w: &mut S) {
        // TODO: Update creds with localhost for using client handler
        w.group(b"TELE", |w| {
            w.tag_str(b"ADRS", "https://localhost:42230");
            w.tag_zero(b"ANON");
            w.tag_str(b"DISA", "AD,AF,AG,AI,AL,AM,AN,AO,AQ,AR,AS,AW,AX,AZ,BA,BB,BD,BF,BH,BI,BJ,BM,BN,BO,BR,BS,BT,BV,BW,BY,BZ,CC,CD,CF,CG,CI,CK,CL,CM,CN,CO,CR,CU,CV,CX,DJ,DM,DO,DZ,EC,EG,EH,ER,ET,FJ,FK,FM,FO,GA,GD,GE,GF,GG,GH,GI,GL,GM,GN,GP,GQ,GS,GT,GU,GW,GY,HM,HN,HT,ID,IL,IM,IN,IO,IQ,IR,IS,JE,JM,JO,KE,KG,KH,KI,KM,KN,KP,KR,KW,KY,KZ,LA,LB,LC,LI,LK,LR,LS,LY,MA,MC,MD,ME,MG,MH,ML,MM,MN,MO,MP,MQ,MR,MS,MU,MV,MW,MY,MZ,NA,NC,NE,NF,NG,NI,NP,NR,NU,OM,PA,PE,PF,PG,PH,PK,PM,PN,PS,PW,PY,QA,RE,RS,RW,SA,SB,SC,SD,SG,SH,SJ,SL,SM,SN,SO,SR,ST,SV,SY,SZ,TC,TD,TF,TG,TH,TJ,TK,TL,TM,TN,TO,TT,TV,TZ,UA,UG,UM,UY,UZ,VA,VC,VE,VG,VN,VU,WF,WS,YE,YT,ZM,ZW,ZZ");
            w.tag_zero(b"EDCT");
            w.tag_str(b"FILT", "-UION/****");
            w.tag_u32(b"LOC", 1701727834);
            w.tag_zero(b"MINR");
            w.tag_str(b"NOOK", "US,CA,MX,NZ");
            w.tag_u16(b"PORT", LOCAL_HTTP_PORT);
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
            // TODO: Update with user id
            w.tag_u32(b"UID", 978651371)
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
