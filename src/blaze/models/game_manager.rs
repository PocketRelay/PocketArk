use std::{net::Ipv4Addr, str::FromStr};

use bitflags::bitflags;
use serde::Serialize;
use tdf::{
    ObjectId, TdfDeserialize, TdfDeserializeOwned, TdfGeneric, TdfMap, TdfSerialize, TdfType,
    TdfTyped,
    types::{string::write_empty_str, tagged_union::TAGGED_UNSET_KEY},
};

use crate::{
    blaze::models::user_sessions::{NatType, PairAddress},
    config::{Config, TunnelConfig},
    database::entity::users::UserId,
    services::{
        game::{AttrMap, Game, GameID, player::GamePlayer},
        tunnel::http_tunnel::TUNNEL_HOST_LOCAL_PORT,
    },
};

use super::user_sessions::NetworkAddress;

#[derive(Debug, Clone)]
#[repr(u32)]
#[allow(unused)]
pub enum GameManagerError {
    InvalidGameId = 0x2,
    GameFull = 0x4,
    PermissionDenied = 0x1e,
    PlayerNotFound = 0x65,
    AlreadyGameMember = 0x67,
    RemovePlayerFailed = 0x68,
    JoinPlayerFailed = 0x6c,
    AlreadyInQueue = 0x70,
    TeamFull = 0xff,
}

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
    pub state: GameState,
}

/// Request to update the state of a mesh connection between
/// payers.
#[derive(TdfDeserialize)]
pub struct MeshEndpointsConnectedRequest {
    #[tdf(tag = "FLGS")]
    pub flags: u32,
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "QOSI")]
    pub qos_info: MeshConnectionQosInfo,
    #[tdf(tag = "TCG")]
    pub target_group_id: ObjectId,
}

#[derive(TdfDeserialize, TdfTyped)]
#[tdf(group)]
pub struct MeshConnectionQosInfo {
    #[tdf(tag = "LOSS")]
    pub packet_loss: f32,
    #[tdf(tag = "PING")]
    pub latency_ms: u32,
}

#[derive(TdfDeserialize, TdfTyped)]
#[repr(u8)]
pub enum PlayerNetConnectionStatus {
    Disconnected = 0x0,
    EstablishingConnection = 0x1,
    Connected = 0x2,
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
        w.tag_owned(b"MSCD", self.pid);
        w.tag_owned(b"MSID", self.pid);
        w.tag_owned(b"USID", self.pid);
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
        #[tdf(tag = "GENT")]
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
    pub config: &'a Config,
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
                w.group(b"PS4", |w| {
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
            w.tag_owned(b"GSET", game.settings.bits());
            w.tag_owned(b"GSID", game.id);
            w.tag_ref(b"GSTA", &game.state);

            w.tag_str_empty(b"GTYP");
            w.tag_str_empty(b"GURL");

            let host_net = host.net().unwrap_or_default();

            // Whether to tunnel the connection
            let tunnel = match &self.config.tunnel {
                TunnelConfig::Stricter => !matches!(host_net.qos.natt, NatType::Open),
                TunnelConfig::Always => true,
                TunnelConfig::Disabled => false,
            };

            {
                w.tag_list_start(b"HNET", TdfType::Group, 1);

                // Override for tunneling
                if tunnel {
                    // Forced local host for test dedicated server
                    w.write_byte(3);
                    TdfSerialize::serialize(
                        &PairAddress {
                            addr: Ipv4Addr::LOCALHOST,
                            port: TUNNEL_HOST_LOCAL_PORT,
                            maci: 0,
                        },
                        w,
                    );
                } else {
                    // Open NATs can directly have players connect normally
                    if let NetworkAddress::AddressPair(pair) = &host_net.addr {
                        w.write_byte(2 /* Address pair type */);
                        TdfSerialize::serialize(pair, w)
                    } else {
                        // Uh oh.. host networking is missing...?
                        w.write_byte(TAGGED_UNSET_KEY);
                        w.write_byte(0);
                    }
                }
            }

            // Max player capacity
            w.tag_u8(b"MCAP", 4);
            // Min player capacity
            w.tag_u8(b"MNCP", 1);
            w.tag_str_empty(b"NPSI");
            w.tag_ref(b"NQOS", &host_net.qos);

            // Flag to indicate that this game is not resetable. This applies only to the CLIENT_SERVER_DEDICATED topology.  The game will be prevented from ever going into the RESETABlE state.
            w.tag_bool(b"NRES", false);
            // The topology used by the game. Typically either client-server, full or partial mesh. Game Groups must set this to NETWORK_DISABLED.
            // w.tag_alt(
            //     b"NTOP",
            //     if tunnel {
            //         GameNetworkTopology::Dedicated
            //     } else {
            //         GameNetworkTopology::PeerHosted
            //     },
            // );
            w.tag_alt(b"NTOP", GameNetworkTopology::PeerHosted);
            w.tag_str_empty(b"PGID");
            w.tag_blob_empty(b"PGSR");

            // The platform speicific host (ie. xbox presence session holder).
            w.tag_ref(
                b"PHST",
                &HostInfo {
                    player_id: host.user.id,
                    connection_group_id: host.user.id,
                    user_session_id: host.user.id,
                    connection_slot_id: 0,
                    slot_id: 0,
                },
            );

            // Presence mode used for 1st party display. May be set to private.
            w.tag_alt(b"PRES", PresenceMode::Standard);

            // Overrides the player reservation timeout for joining players.  (Joining Scenarios can override this.)
            w.tag_u8(b"PRTO", 0);

            // Ping site alias
            w.tag_str(b"PSAS", "bio-dub");

            // Is pseudo game
            w.tag_bool(b"PSEU", false);

            // Queue capacity
            w.tag_u8(b"QCAP", 0);

            // The roles and capacities, and criteria, supported in this game session
            w.group(b"RNFO", |w| {
                w.tag_map_start(b"CRIT", TdfType::String, TdfType::Group, 1);
                write_empty_str(w);
                w.group_body(|w| {
                    w.tag_u8(b"RCAP", 4);
                });
            });

            // External Session service config identifier
            w.tag_str_empty(b"SCID");

            // 32 bit number shared between clients (Should this be randomized?)
            w.tag_u32(b"SEED", 2096547478);
            w.tag_str_empty(b"STMN");

            // The topology host for the game (everyone connects to this person).
            w.tag_ref(
                b"THST",
                &HostInfo {
                    player_id: host.user.id,
                    connection_group_id: host.user.id,
                    user_session_id: host.user.id,
                    connection_slot_id: 0,
                    slot_id: 0,
                },
            );

            // Team ID vector
            w.tag_list_slice(b"TIDS", &[65534]);
            w.tag_str(b"UUID", "add98ceb-1ea3-40ad-9c9d-8c855fedf6ef");
            w.tag_alt(b"VOIP", VoipTopology::PeerToPeer);
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
        w.tag_u32(b"TELM", 20000000);
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

pub struct NotifyMatchmakingSessionConnectionValidated {
    pub player_id: u32,
    pub game_id: u32,
}

impl TdfSerialize for NotifyMatchmakingSessionConnectionValidated {
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
    pub state: GameState,
}

#[derive(TdfSerialize)]
pub struct ReportingIdChange {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "GRID")]
    pub grid: u64,
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

#[derive(TdfSerialize)]
pub struct PlayerStateChange {
    #[tdf(tag = "GID")]
    pub gid: GameID,
    #[tdf(tag = "PID")]
    pub pid: UserId,
    #[tdf(tag = "STAT")]
    pub state: PlayerState,
}

#[derive(
    Default, Debug, Serialize, Clone, Copy, PartialEq, Eq, TdfDeserialize, TdfSerialize, TdfTyped,
)]
#[repr(u8)]
pub enum PlayerState {
    /// Link between the mesh points is not connected
    #[default]
    #[tdf(default)]
    Reserved = 0x0,
    Queued = 0x1,
    /// Link is being formed between two mesh points
    ActiveConnecting = 0x2,
    ActiveMigrating = 0x3,
    /// Link is connected between two mesh points
    ActiveConnected = 0x4,
    ActiveKickPending = 0x5,
}

#[derive(TdfSerialize)]
pub struct JoinComplete {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "PID")]
    pub player_id: UserId,
}

pub struct AsyncMatchmakingStatus {
    pub user_id: UserId,
}

#[derive(TdfSerialize, TdfTyped)]
#[tdf(group)]
struct GameAttributeRuleStatus {
    #[tdf(tag = "NAME")]
    name: String,
    #[tdf(tag = "VALU")]
    value: Vec<String>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct EvaluateStatus: u16 {
        const PLAYER_COUNT_SUFFICIENT = 0x1;
        const ACCEPTABLE_HOST_FOUND = 0x2;
        const TEAM_SIZES_SUFFICIENT = 0x4;
    }
}

impl From<u16> for EvaluateStatus {
    fn from(value: u16) -> Self {
        Self::from_bits_retain(value)
    }
}

impl From<EvaluateStatus> for u16 {
    fn from(value: EvaluateStatus) -> Self {
        value.bits()
    }
}

#[derive(TdfSerialize, TdfTyped)]
#[tdf(group)]
struct CreateGameStatus {
    #[tdf(tag = "EVST", into = u16)]
    evaluate_state: EvaluateStatus,
    #[tdf(tag = "MMSN")]
    number_of_matchmaking_session: u16,
    #[tdf(tag = "NOMP")]
    number_of_matched_players: u16,
}

#[derive(TdfSerialize, TdfTyped)]
#[tdf(group)]
struct FindGameStatus {
    #[tdf(tag = "GNUM")]
    number_of_games: u16,
}

impl TdfSerialize for AsyncMatchmakingStatus {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_list_start(b"ASIL", TdfType::Group, 1);
        w.group_body(|w| {
            w.tag_ref(
                b"CGS",
                &CreateGameStatus {
                    evaluate_state: EvaluateStatus::PLAYER_COUNT_SUFFICIENT
                        | EvaluateStatus::ACCEPTABLE_HOST_FOUND
                        | EvaluateStatus::TEAM_SIZES_SUFFICIENT,
                    number_of_matchmaking_session: 1,
                    number_of_matched_players: 0,
                },
            );

            w.tag_ref(b"FGS", &FindGameStatus { number_of_games: 0 });

            w.tag_map_tuples(
                b"GASM",
                &[
                    (
                        "gameDifficultyRule".to_string(),
                        GameAttributeRuleStatus {
                            name: "gameDifficultyRule".to_string(),
                            value: vec!["4".to_string()],
                        },
                    ),
                    (
                        "gameEnemyTypeRule".to_string(),
                        GameAttributeRuleStatus {
                            name: "gameEnemyTypeRule".to_string(),
                            value: vec!["3".to_string()],
                        },
                    ),
                    (
                        "gameLevelNameRule".to_string(),
                        GameAttributeRuleStatus {
                            name: "gameLevelNameRule".to_string(),
                            value: vec!["13".to_string()],
                        },
                    ),
                    (
                        "gameMissionSlotRule".to_string(),
                        GameAttributeRuleStatus {
                            name: "gameMissionSlotRule".to_string(),
                            value: vec![
                                "0".to_string(),
                                "1".to_string(),
                                "2".to_string(),
                                "3".to_string(),
                                "4".to_string(),
                                "5".to_string(),
                                "6".to_string(),
                                "7".to_string(),
                                "8".to_string(),
                                "9".to_string(),
                            ],
                        },
                    ),
                ],
            );

            // Geo location rule status
            w.group(b"GEOS", |w| {
                // Max distance
                w.tag_zero(b"DIST");
            });

            // Host balance rule status
            w.group(b"HBRD", |w| {
                // Host balance values
                // HOSTS_STRICTLY_BALANCED = 0,
                // HOSTS_BALANCED = 1,
                // HOSTS_UNBALANCED = 2,

                w.tag_u8(b"BVAL", 2);
            });

            // Host viability rule status
            w.group(b"HVRD", |w| {
                // Host viability values
                // CONNECTION_ASSURED = 0,
                // CONNECTION_LIKELY = 1,
                // CONNECTION_FEASIBLE = 2,
                // CONNECTION_UNLIKELY = 3,

                w.tag_u8(b"VVAL", 1);
            });

            // Unknown
            w.group(b"PLCN", |w| {
                w.tag_u8(b"PMAX", 4);
                w.tag_u8(b"PMIN", 4);
            });

            w.group(b"PLUT", |w| {
                w.tag_u8(b"PMAX", 0);
                w.tag_u8(b"PMIN", 0);
            });

            // Ping site rule status
            w.group(b"PSRS", |w| {
                w.tag_list_slice(b"VALU", &["bio-dub"]);
            });

            // Rank rule status
            w.group(b"RRDA", |w| {
                // Matched rank flags
                w.tag_zero(b"RVAL");
            });

            // Unknown
            w.group(b"TBRS", |w| {
                w.tag_zero(b"SDIF");
            });

            // Unknown
            w.group(b"TCPS", |w| {
                w.tag_str_empty(b"NAME");
            });

            // Unknown
            w.group(b"TMSS", |w| {
                w.tag_zero(b"PCNT");
            });

            // Unknown
            w.group(b"TOTS", |w| {
                w.tag_u8(b"PMAX", 0);
                w.tag_u8(b"PMIN", 0);
            });

            // Unknown
            w.group(b"TPPS", |w| {
                w.tag_zero(b"BDIF");
                w.tag_zero(b"BOTN");
                w.tag_str_empty(b"NAME");
                w.tag_zero(b"TDIF");
                w.tag_zero(b"TOPN");
            });

            // Unknown
            w.group(b"TUBS", |w| {
                w.tag_zero(b"MUED");
                w.tag_str_empty(b"NAME");
                w.tag_zero(b"SDIF");
            });

            // Virtual game rule status
            w.group(b"VGRS", |w| {
                w.tag_u8(b"VVAL", 3);
            });
        });

        w.tag_owned(b"MSCD", self.user_id);
        w.tag_owned(b"MSID", self.user_id);
        w.tag_owned(b"USID", self.user_id);
    }
}

/// Message for a player joining notification
pub struct PlayerJoining<'a> {
    /// The ID of the game
    pub game_id: GameID,
    /// The slot the player is joining into
    pub slot: usize,
    /// The player that is joining
    pub player: &'a GamePlayer,
}

impl TdfSerialize for PlayerJoining<'_> {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_u32(b"GID", self.game_id);

        w.tag_group(b"PDAT");
        self.player.encode(self.game_id, self.slot, w);
    }
}

#[derive(TdfSerialize)]
pub struct AdminListChange {
    #[tdf(tag = "ALST")]
    pub player_id: UserId,
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "OPER")]
    pub operation: AdminListOperation,
    #[tdf(tag = "UID")]
    pub host_id: UserId,
}

/// Different operations that can be performed on
/// the admin list
#[derive(Debug, Clone, Copy, TdfSerialize, TdfTyped)]
#[repr(u8)]
pub enum AdminListOperation {
    Add = 0,
    Remove = 1,
}

#[derive(TdfDeserialize)]
pub struct AddAdminPlayerRequest {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "PID")]
    pub player_id: UserId,
}

#[derive(TdfDeserialize)]
pub struct SetSettingRequest {
    #[tdf(tag = "GID")]
    pub game_id: GameID,
    #[tdf(tag = "GSET", into = u32)]
    pub setting: GameSettings,
}

/// Message for a game setting changing
#[derive(TdfSerialize)]
pub struct SettingChange {
    /// The game setting
    #[tdf(tag = "ATTR", into = u32)]
    pub settings: GameSettings,
    /// The ID of the game
    #[tdf(tag = "GID")]
    pub id: u32,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct GameSettings: u32 {
        const NONE = 0;
        const OPEN_TO_BROWSING = 1;
        const OPEN_TO_MATCHMAKING = 2;
        const OPEN_TO_INVITES = 4;
        const OPEN_TO_JOIN_BY_PLAYER = 8;
        const HOST_MIGRATABLE = 0x10;
        const RANKED = 0x20;
        const ADMIN_ONLY_INVITES = 0x40;
        const ENFORCE_SINGLE_GROUP_JOIN = 0x80;
        const JOIN_IN_PROGRESS_SUPPORTED = 0x100;
        const ADMIN_INVITE_ONLY_IGNORE_ENTRY_CHECKS = 0x200;
        const IGNORE_ENTRY_CRITERIA_WITH_INVITE = 0x400;
        const ENABLE_PERSISTED_GAME_ID = 0x800;
        const ALLOW_SAME_TEAM_ID = 0x1000;
        const VIRTUALIZED = 0x2000;
        const SEND_ORPHANED_GAME_REPORT_EVENT = 0x4000;
        const ALLOW_ANY_REPUTATION = 0x8000;
        const LOCKED_AS_BUSY = 0x10000;
        const DISCONNECT_RESERVATION = 0x20000;
        const DYNAMIC_REPUTATION_REQUIREMENT = 0x40000;
        const FRIENDS_BYPASS_CLOSED_TO_JOIN_BY_PLAYER = 0x80000;
        const ALLOW_MEMBER_GAME_ATTRIBUTE_EDIT = 0x100000;
        const AUTO_DEMOTE_RESERVED_PLAYERS = 0x200000;
        const UPDATE_QUEUE_CAPACITY_ON_RESET = 0x400000;
        const SPECTATOR_BYPASS_CLOSED_TO_JOIN = 0x800000;
    }
}

impl From<GameSettings> for u32 {
    fn from(value: GameSettings) -> Self {
        value.bits()
    }
}

impl From<u32> for GameSettings {
    fn from(value: u32) -> Self {
        GameSettings::from_bits_retain(value)
    }
}

/// Different states the game can be in
#[derive(
    Default, Debug, Serialize, Clone, Copy, PartialEq, Eq, TdfSerialize, TdfDeserialize, TdfTyped,
)]
#[repr(u8)]
pub enum GameState {
    /// Data structure just created
    NewState = 0x0,
    /// Closed to joins/matchmaking
    #[tdf(default)]
    #[default]
    Initializing = 0x1,
    /// Game will need topology host assigned when player joins.
    InactiveVirtual = 0x2,
    /// Game created via matchmaking is waiting for connections to be established and validated.
    ConnectionVerification = 0x3,
    /// Pre game state, obey joinMode flags
    PreGame = 0x82,
    /// Game available, obey joinMode flag
    InGame = 0x83,
    /// After game is done,closed to joins/matchmaking
    PostGame = 0x4,
    /// Game migration state, closed to joins/matchmaking
    Migrating = 0x5,
    /// Game destruction state, closed to joins/matchmaking
    Destructing = 0x6,
    /// Game resettable state, closed to joins/matchmaking, but available to be reset
    Resettable = 0x7,
    /// Unresponsive, closed to joins/matchmaking
    Unresponsive = 0x9,
    /// Initialized state, intended for the use of game group
    GameGroupInitialized = 0x10,
}

#[derive(TdfSerialize, TdfTyped)]
#[tdf(group)]
pub struct HostInfo {
    #[tdf(tag = "CONG")]
    connection_group_id: u32,
    #[tdf(tag = "CSID")]
    connection_slot_id: u32,
    #[tdf(tag = "HPID")]
    player_id: UserId,
    #[tdf(tag = "HSES")]
    user_session_id: UserId,
    #[tdf(tag = "HSLT")]
    slot_id: u8,
}
