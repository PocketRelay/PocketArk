use std::str::FromStr;

use tdf::{
    types::string::write_empty_str, ObjectId, TdfDeserialize, TdfDeserializeOwned, TdfMap,
    TdfSerialize, TdfType, TdfTyped, U12,
};

use crate::{
    database::entity::users::UserId,
    services::game::{AttrMap, Game, GameID},
};

use super::user_sessions::NetworkAddress;

#[derive(TdfDeserialize)]
pub struct MatchmakeRequest {
    #[tdf(tag = "SCNA")]
    pub attributes: TdfMap<String, U12>,
    #[tdf(tag = "SCNM", into = &str)]
    pub ty: MatchmakeType,
}

pub enum MatchmakeType {
    QuickMatch,       // standardQuickMatch
    CreatePublicGame, // createPublicGame
}

impl From<&str> for MatchmakeType {
    fn from(value: &str) -> Self {
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

#[derive(TdfSerialize, TdfTyped)]
pub enum GameSetupContext {
    /// Context without additional data
    #[tdf(key = 0x0, tag = "VALU")]
    Dataless {
        #[tdf(tag = "DCTX")]
        context: DatalessContext,
    },
    /// Context added from matchmaking
    #[tdf(key = 0x3, tag = "VALU")]
    Matchmaking {
        #[tdf(tag = "FIT")]
        fit_score: u16,
        #[tdf(tag = "MAXF")]
        max_fit_score: u16,
        #[tdf(tag = "MSCD")]
        id_1: UserId,
        #[tdf(tag = "MSID")]
        id_2: UserId,
        #[tdf(tag = "RSLT")]
        result: MatchmakingResult,
        #[tdf(tag = "TOUT")]
        tout: u32,
        #[tdf(tag = "TTM")]
        ttm: u32,
        #[tdf(tag = "USID")]
        id_3: UserId,
    },
}

#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum MatchmakingResult {
    CreatedGame = 0x0,
    JoinedNewGame = 0x1,
    JoinedExistingGame = 0x2,
    // TimedOut = 0x3,
    // Canceled = 0x4,
    // Terminated = 0x5,
    // GameSetupFailed = 0x6,
}

#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum DatalessContext {
    /// Session created the game
    CreateGameSetup = 0x0,
    /// Session joined by ID
    JoinGameSetup = 0x1,
    // IndirectJoinGameFromQueueSetup = 0x2,
    // IndirectJoinGameFromReservationContext = 0x3,
    // HostInjectionSetupContext = 0x4,
}

pub struct GameSetupResponse<'a> {
    pub game: &'a Game,
    pub context: GameSetupContext,
}

impl TdfSerialize for GameSetupResponse<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        let game = self.game;
        let host = game.players.first().expect("Missing game host for setup");

        w.group(b"GAME", |w| {
            w.tag_list_iter_owned(b"ADMN", game.players.iter().map(|player| player.user.id));
            w.tag_u8(b"APRS", 1);
            w.tag_ref(b"ATTR", &game.attributes);
            w.tag_list_slice::<u8>(b"CAP", &[4, 0, 0, 0]);
            w.tag_u8(b"CCMD", 3);
            w.tag_str_empty(b"COID");
            w.tag_str_empty(b"CSID");
            w.tag_u64(b"CTIM", 1688851953868334);
            w.group(b"DHST", |w| {
                w.tag_zero(b"CONG");
                w.tag_zero(b"CSID");
                w.tag_zero(b"HPID");
                w.tag_zero(b"HSES");
                w.tag_zero(b"HSLT");
            });
            w.tag_zero(b"DRTO");
            w.group(b"ESID", |w| {
                w.group(b"PS\x20", |w| {
                    w.tag_str_empty(b"NPSI");
                });
                w.group(b"XONE", |w| {
                    w.tag_str_empty(b"COID");
                    w.tag_str_empty(b"ESNM");
                    w.tag_str_empty(b"STMN");
                });
            });

            w.tag_str_empty(b"ESNM");
            w.tag_zero(b"GGTY");

            w.tag_u32(b"GID", game.id);
            w.tag_zero(b"GMRG");
            w.tag_str_empty(b"GNAM");
            w.tag_u64(b"GPVH", 0x5a4f2b378b715c6);
            w.tag_owned(b"GSET", game.settings);
            w.tag_owned(b"GSID", game.id);
            w.tag_ref(b"GSTA", &game.state);

            w.tag_str_empty(b"GTYP");
            w.tag_str_empty(b"GURL");
            {
                w.tag_list_start(b"HNET", TdfType::Group, 1);
                w.write_byte(2);
                if let NetworkAddress::AddressPair(pair) = &host.net.addr {
                    TdfSerialize::serialize(pair, w)
                }
            }
            w.tag_u8(b"MCAP", 1); // should be 4?
            w.tag_u8(b"MNCP", 1);
            w.tag_str_empty(b"NPSI");
            w.tag_ref(b"NQOS", &host.net.qos);

            w.tag_zero(b"NRES");
            w.tag_zero(b"NTOP");
            w.tag_str_empty(b"PGID");
            w.tag_blob_empty(b"PGSR");

            w.group(b"PHST", |w| {
                w.tag_u32(b"CONG", host.user.id);
                w.tag_u32(b"CSID", 0);
                w.tag_u32(b"HPID", host.user.id);
                w.tag_zero(b"HSLT");
            });
            w.tag_u8(b"PRES", 0x1);
            w.tag_u8(b"PRTO", 0);
            w.tag_str(b"PSAS", "bio-syd");
            w.tag_u8(b"PSEU", 0);
            w.tag_u8(b"QCAP", 0);
            w.group(b"RNFO", |w| {
                w.tag_map_start(b"CRIT", TdfType::String, TdfType::Group, 1);
                write_empty_str(w);
                w.group_body(|w| {
                    w.tag_u8(b"RCAP", 1);
                });
            });
            w.tag_str_empty(b"SCID");
            w.tag_u32(b"SEED", 131492528);
            w.tag_str_empty(b"STMN");

            w.group(b"THST", |w| {
                w.tag_u32(b"CONG", host.user.id);
                w.tag_u8(b"CSID", 0x0);
                w.tag_u32(b"HPID", host.user.id);
                w.tag_u32(b"HSES", host.user.id);
                w.tag_u8(b"HSLT", 0x0);
            });

            w.tag_list_slice(b"TIDS", &[65534]);
            w.tag_str(b"UUID", "32d89cf8-6a83-4282-b0a0-5b7a8449de2e");
            w.tag_u8(b"VOIP", 0);
            w.tag_str(b"VSTR", "60-Future739583");
        });

        // Player list
        w.tag_list_start(b"PROS", TdfType::Group, game.players.len());
        for (slot, player) in game.players.iter().enumerate() {
            player.encode(game.id, slot, w);
        }

        w.group(b"QOSS", |w| {
            w.tag_u8(b"DURA", 0);
            w.tag_u8(b"INTV", 0);
            w.tag_u8(b"SIZE", 0);
        });

        w.tag_u8(b"QOSV", 0);

        w.tag_ref(b"REAS", &self.context);
    }
}

#[derive(TdfSerialize)]
pub struct PlayerRemoved {
    #[tdf(tag = "CNTX")]
    pub cntx: u32,
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "PID")]
    pub player_id: u32,
    #[tdf(tag = "REAS")]
    pub reason: RemoveReason,
}

#[derive(Debug, Clone, Copy, TdfDeserialize, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum RemoveReason {
    /// Hit timeout while joining
    JoinTimeout = 0x0,
    /// Player lost PTP conneciton
    PlayerConnectionLost = 0x1,
    /// Player lost connection with the Pocket Relay server
    ServerConnectionLost = 0x2,
    /// Game migration failed
    MigrationFailed = 0x3,
    GameDestroyed = 0x4,
    GameEnded = 0x5,
    /// Generic player left the game reason
    #[tdf(default)]
    PlayerLeft = 0x6,
    GroupLeft = 0x7,
    /// Player kicked
    PlayerKicked = 0x8,
    /// Player kicked and banned
    PlayerKickedWithBan = 0x9,
    /// Failed to join from the queue
    PlayerJoinFromQueueFailed = 0xA,
    PlayerReservationTimeout = 0xB,
    HostEjected = 0xC,
}

pub struct NotifyPostJoinedGame {
    pub player_id: u32,
    pub game_id: u32,
}

impl TdfSerialize for NotifyPostJoinedGame {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        // TODO: Something to do with matchmaking?
        w.group(b"CONV", |w| {
            w.tag_zero(b"FCNT");
            w.tag_zero(b"NTOP");
            w.tag_zero(b"TIER");
        });
        w.tag_u8(b"DISP", 1);
        w.tag_owned(b"GID", self.game_id);

        w.tag_alt(b"GRID", ObjectId::new_raw(0, 0, 0));

        w.tag_owned(b"MSCD", self.player_id);
        w.tag_owned(b"MSID", self.player_id);
        w.tag_zero(b"QSVR");
        w.tag_owned(b"USID", self.player_id);
    }
}

#[derive(TdfSerialize)]
pub struct NotifyGameStateChange {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "GSTA")]
    pub state: u8,
}

#[derive(TdfSerialize)]
pub struct NotifyGameReplay {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "GRID")]
    pub grid: GameID,
}

/// Packet for game attribute changes
pub struct AttributesChange<'a> {
    /// Borrowed game attributes map
    pub attributes: &'a AttrMap,
    /// The id of the game the attributes have changed for
    pub id: GameID,
}

impl TdfSerialize for AttributesChange<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_ref(b"ATTR", self.attributes);
        w.tag_owned(b"GID", self.id);
    }
}

/// Packet for game attribute changes
pub struct PlayerAttributesChange<'a> {
    /// Borrowed game attributes map
    pub attributes: &'a AttrMap,
    pub game_id: GameID,
    pub user_id: UserId,
}

impl TdfSerialize for PlayerAttributesChange<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_ref(b"ATTR", self.attributes);
        w.tag_owned(b"GID", self.game_id);
        w.tag_owned(b"PID", self.user_id);
    }
}
