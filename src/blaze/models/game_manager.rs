use std::str::FromStr;

use tdf::{TdfDeserialize, TdfDeserializeOwned, TdfSerialize, TdfType};

use crate::services::game::{AttrMap, RemoveReason};

pub struct MatchmakeRequest {
    pub ty: MatchmakeType,
}

impl TdfDeserializeOwned for MatchmakeRequest {
    fn deserialize_owned(r: &mut tdf::TdfDeserializer<'_>) -> tdf::DecodeResult<Self> {
        let value: String = r.tag(b"SCNM")?;
        let ty = MatchmakeType::parse(&value);
        Ok(Self { ty })
    }
}

pub enum MatchmakeType {
    QuickMatch,       // standardQuickMatch
    CreatePublicGame, // createPublicGame
}

impl MatchmakeType {
    pub fn parse(value: &str) -> Self {
        match value {
            "standardQuickMatch" => Self::QuickMatch,
            _ => Self::CreatePublicGame,
        }
    }
}
pub struct MatchmakingResponse {
    pub user_id: u32,
}

impl TdfSerialize for MatchmakingResponse {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_str_empty(b"COID");
        w.tag_str_empty(b"ESNM");
        w.tag_owned(b"MSID", self.user_id);
        w.tag_str_empty(b"SCID");
        w.tag_str_empty(b"STMN");
    }
}

#[derive(TdfDeserialize)]
pub struct UpdateGameAttrRequest {
    #[tdf(tag = "ATTR")]
    pub attr: AttrMap,
    #[tdf(tag = "GID")]
    pub gid: u32,
}

#[derive(TdfDeserialize)]
pub struct UpdateAttrRequest {
    #[tdf(tag = "ATTR")]
    pub attr: AttrMap,
    #[tdf(tag = "GID")]
    pub gid: u32,
    #[tdf(tag = "PID")]
    pub pid: u32,
}

#[derive(TdfDeserialize)]
pub struct UpdateStateRequest {
    #[tdf(tag = "GID")]
    pub gid: u32,
    #[tdf(tag = "GSTA")]
    pub state: u8,
}

#[derive(TdfDeserialize)]
pub struct ReplayGameRequest {
    #[tdf(tag = "GID")]
    pub gid: u32,
}

#[derive(TdfDeserialize)]
pub struct LeaveGameRequest {
    #[tdf(tag = "GID")]
    pub gid: u32,
    #[tdf(tag = "REAS")]
    pub reas: RemoveReason,
}

pub struct NotifyMatchmakingStatus {
    pub pid: u32,
}

impl TdfSerialize for NotifyMatchmakingStatus {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        {
            w.tag_list_start(b"ASIL", TdfType::Group, 1);
            w.group_body(|w| {
                w.group(b"CGS", |w| {
                    w.tag_u8(b"EVST", 0);
                    w.tag_u8(b"MMSN", 1);
                    w.tag_u8(b"NOMP", 0);
                });

                w.group(b"FGS", |w| w.tag_u8(b"GNUM", 0));
                w.group(b"GEOS", |w| w.tag_u8(b"DIST", 0));
                w.group(b"HBRD", |w| w.tag_u8(b"BVAL", 0));
                w.group(b"HVRD", |w| w.tag_u8(b"VVAL", 0));
                w.group(b"PLCN", |w| {
                    w.tag_u8(b"PMAX", 1);
                    w.tag_u8(b"PMIN", 1);
                });
                w.group(b"PLUT", |w| {
                    w.tag_u8(b"PMAX", 0);
                    w.tag_u8(b"PMIN", 0);
                });
                w.tag_group_empty(b"PSRS");
                w.group(b"RRDA", |w| w.tag_u8(b"RVAL", 0));
                w.group(b"TBRS", |w| w.tag_u8(b"SDIF", 0));
                w.group(b"TCPS", |w| w.tag_str_empty(b"NAME"));
                w.group(b"TMSS", |w| w.tag_u8(b"PCNT", 0));
                w.group(b"TOTS", |w| {
                    w.tag_u8(b"PMAX", 4);
                    w.tag_u8(b"PMIN", 4);
                });
                w.group(b"TPPS", |w| {
                    w.tag_u8(b"BDIF", 0);
                    w.tag_u8(b"BOTN", 0);
                    w.tag_str_empty(b"NAME");
                    w.tag_u8(b"TDIF", 0);
                    w.tag_u8(b"TOPN", 0);
                });
                w.group(b"TPPS", |w| {
                    w.tag_u8(b"MUED", 0);
                    w.tag_str_empty(b"NAME");
                    w.tag_u8(b"SDIF", 0);
                });
                w.group(b"VGRS", |w| w.tag_u8(b"VVAL", 0));
            });
        }
        w.tag_owned(b"MSCD", self.pid); // pid
        w.tag_owned(b"MSID", self.pid); // pid
        w.tag_owned(b"USID", self.pid); // pid
    }
}
