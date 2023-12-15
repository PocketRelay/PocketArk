use std::str::FromStr;

use tdf::{
    types::string::write_empty_str, ObjectId, TdfDeserialize, TdfDeserializeOwned, TdfGeneric,
    TdfMap, TdfSerialize, TdfType, TdfTyped,
};

use crate::{
    database::entity::users::UserId,
    services::game::{AttrMap, Game, GameID},
};

use super::user_sessions::NetworkAddress;

#[derive(TdfDeserialize)]
pub struct StartMatchmakingScenarioRequest {
    #[tdf(tag = "SCNA")]
    pub attributes: TdfMap<String, TdfGeneric>,
    #[tdf(tag = "SCNM", into = &str)]
    pub ty: MatchmakeScenario,
}

pub enum MatchmakeScenario {
    QuickMatch,       // standardQuickMatch
    CreatePublicGame, // createPublicGame
}

impl From<&str> for MatchmakeScenario {
    fn from(value: &str) -> Self {
        match value {
            "standardQuickMatch" => Self::QuickMatch,
            _ => Self::CreatePublicGame,
            // TODO: Handle unknown properly
        }
    }
}

pub struct StartMatchmakingScenarioResponse {
    pub user_id: u32,
}

impl TdfSerialize for StartMatchmakingScenarioResponse {
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
    #[tdf(key = 0x0, tag = "MMSC")]
    Dataless {
        #[tdf(tag = "DCTX")]
        context: DatalessContext,
    },
    /// Context added from matchmaking
    #[tdf(key = 0x3, tag = "MMSC")]
    Matchmaking {
        #[tdf(tag = "FIT")]
        fit_score: u16,
        #[tdf(tag = "FIT")]
        fit_score_2: u16,
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
    TimedOut = 0x3,
    Canceled = 0x4,
    Terminated = 0x5,
    GameSetupFailed = 0x6,
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

#[allow(unused)]
#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum PresenceMode {
    // No presence management. E.g. For games that should never be advertised in shell UX and cannot be used for 1st party invites.
    None = 0x0,
    // Full presence as defined by the platform.
    Standard = 0x1,
    // Private presence as defined by the platform. For private games which are closed to uninvited/outside users.
    Private = 0x2,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum VoipTopology {
    /// VOIP is disabled (for a game)
    Disabled = 0x0,
    // /// VOIP uses a star topology; typically some form of 3rd party server dedicated to mixing/broadcasting voip streams.
    // DedicatedServer = 0x1
    /// VOIP uses a full mesh topology; each player makes peer connections to the other players/members for voip traffic.
    PeerToPeer = 0x2,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum GameNetworkTopology {
    /// client server peer hosted network topology
    PeerHosted = 0x0,
    /// client server dedicated server topology
    Dedicated = 0x1,
    /// Peer to peer full mesh network topology
    FullMesh = 0x82,
    /// Networking is disabled??
    Disabled = 0xFF,
}

/// Various modes that the game can be configured to leverage Connection Concierge service (CCS).
#[allow(unused)]
#[derive(Debug, Copy, Clone, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum CCSMode {
    /// Invalid value.
    Invalid = 0x0,
    /// No connections are attempted via the CCS(acts as disabled).
    PeerOnly = 0x1,
    /// Connections are attempted via the CCS only(used for testing).
    HostedOnly = 0x2,
    /// CCS is used for making failed connections.
    HostedFallback = 0x3,
}

const GAME_PROTOCOL_VERSION: &str = "60-Future739583";

/// UNSPECIFIED_TEAM_INDEX will assign the player to whichever team has room.
pub const UNSPECIFIED_TEAM_INDEX: u16 = 0xffff;

pub struct GameSetupResponse<'a> {
    pub game: &'a Game,
    pub context: GameSetupContext,
}

impl TdfSerialize for GameSetupResponse<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        let game = self.game;
        let host = game.players.first().expect("Missing game host for setup");

        w.group(b"GAME", |w| {
            // Admin player list
            w.tag_list_iter_owned(b"ADMN", game.players.iter().map(|player| player.user.id));
            // This boolean flag determines if a game session owns first party presence on the client.
            w.tag_bool(b"APRS", true);
            // Game attributes
            w.tag_ref(b"ATTR", &game.attributes);
            // Slot capacities
            w.tag_list_slice::<usize>(
                b"CAP",
                &[
                    Game::MAX_PLAYERS, /* Public slots */
                    0,                 /* Private Slots */
                    0,
                    0,
                ],
            );
            w.tag_alt(b"CCMD", CCSMode::HostedFallback);
            w.tag_str_empty(b"COID");
            w.tag_str_empty(b"CSID");

            // Creation time
            w.tag_u64(b"CTIM", 1688851953868334);

            // The dedicated server host for the game, if there is one. (For non-failover, will be the same as mTopologyHostInfo).
            w.group(b"DHST", |w| {
                w.tag_zero(b"CONG");
                w.tag_zero(b"CSID");
                w.tag_zero(b"HPID");
                w.tag_zero(b"HSES");
                w.tag_zero(b"HSLT");
            });

            // Overrides the player reservation timeout for disconnected players.
            w.tag_zero(b"DRTO");

            // External Session identification.
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
            w.tag_u8(b"GGTY", 0);

            w.tag_u32(b"GID", game.id);
            w.tag_zero(b"GMRG");
            w.tag_str_empty(b"GNAM");
            w.tag_u64(b"GPVH", 3788120962);
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

            // Max player capacity
            w.tag_u8(b"MCAP", 1); // should be 4?
                                  // Min player capacity
            w.tag_u8(b"MNCP", 1);
            w.tag_str_empty(b"NPSI");
            w.tag_ref(b"NQOS", &host.net.qos);

            // Flag to indicate that this game is not resetable. This applies only to the CLIENT_SERVER_DEDICATED topology.  The game will be prevented from ever going into the RESETABlE state.
            w.tag_bool(b"NRES", false);
            // The topology used by the game. Typically either client-server, full or partial mesh. Game Groups must set this to NETWORK_DISABLED.
            w.tag_alt(b"NTOP", GameNetworkTopology::PeerHosted);
            w.tag_str_empty(b"PGID");
            w.tag_blob_empty(b"PGSR");

            // The platform speicific host (ie. xbox presence session holder).
            w.group(b"PHST", |w| {
                w.tag_u32(b"CONG", host.user.id);
                w.tag_u32(b"CSID", 0);
                w.tag_u32(b"HPID", host.user.id);
                w.tag_zero(b"HSLT");
            });

            // Presence mode used for 1st party display. May be set to private.
            w.tag_alt(b"PRES", PresenceMode::Standard);

            // Overrides the player reservation timeout for joining players.  (Joining Scenarios can override this.)
            w.tag_u8(b"PRTO", 0);

            // Ping site alias
            w.tag_str(b"PSAS", "bio-syd");

            // Is pseudo game
            w.tag_bool(b"PSEU", false);

            // Queue capacity
            w.tag_u8(b"QCAP", 0);

            // The roles and capacities, and criteria, supported in this game session
            w.group(b"RNFO", |w| {
                w.tag_map_start(b"CRIT", TdfType::String, TdfType::Group, 1);
                write_empty_str(w);
                w.group_body(|w| {
                    w.tag_u8(b"RCAP", 1);
                });
            });

            // External Session service config identifier
            w.tag_str_empty(b"SCID");

            // 32 bit number shared between clients (Should this be randomized?)
            w.tag_u32(b"SEED", 131492528);
            w.tag_str_empty(b"STMN");

            // The topology host for the game (everyone connects to this person).
            w.group(b"THST", |w| {
                w.tag_u32(b"CONG", host.user.id);
                w.tag_u8(b"CSID", 0x0);
                w.tag_u32(b"HPID", host.user.id);
                w.tag_u32(b"HSES", host.user.id);
                w.tag_u8(b"HSLT", 0x0);
            });

            // Team ID vector
            w.tag_list_slice(b"TIDS", &[65534]);
            w.tag_str(b"UUID", "32d89cf8-6a83-4282-b0a0-5b7a8449de2e");
            w.tag_alt(b"VOIP", VoipTopology::Disabled);
            w.tag_str(b"VSTR", GAME_PROTOCOL_VERSION);
        });

        // Lockable for preferred joins
        w.tag_bool(b"LFPJ", false);

        // mGameModeAttributeName
        w.tag_str(b"MNAM", "coopGameVisibility");

        // Player list
        w.tag_list_start(b"PROS", TdfType::Group, game.players.len());
        for (slot, player) in game.players.iter().enumerate() {
            player.encode(game.id, slot, w);
        }

        // QoS settings
        w.group(b"QOSS", |w| {
            w.tag_u8(b"DURA", 0);
            w.tag_u8(b"INTV", 0);
            w.tag_u8(b"SIZE", 0);
        });

        // If true, the client will perform QoS validation when initializing the network.
        w.tag_bool(b"QOSV", false);

        // Game setup reason
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
        // A new set of connection validation results to store for this user session.
        w.group(b"CONV", |w| {
            // New count of matchmaking finalization failures due to connection issues.
            w.tag_zero(b"FCNT");
            // The network topology that this avoid list applies to.
            w.tag_zero(b"NTOP");
            // Matchmaking QoS evaluation tier, the tier determines what the maximum allowed latency and packet loss are.
            w.tag_zero(b"TIER");
        });
        // If true, the client SDK should dispatch GameManagerAPIListener::onMatchmakingSessionFinished(), if false, the connection validation failed, and the game will be cleaned up silently.
        w.tag_bool(b"DISP", true);
        // The Game Id that was matched.
        w.tag_owned(b"GID", self.game_id);

        // The user group id related to the matchmaking session, required to dispatch to group memebers.
        w.tag_alt(b"GRID", ObjectId::new_raw(0, 0, 0));

        // The matchmaking scenario id.
        w.tag_owned(b"MSCD", self.player_id);
        // The matchmaking session id.
        w.tag_owned(b"MSID", self.player_id);
        // Whether qos validation was performed (qos validation is performed only if there is an applicable qos validation rule configured for the game network topology)
        w.tag_bool(b"QSVR", false);
        // The usersession id of the matchmaking session.
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
