use interlink::{
    prelude::{Handler, Link, Message},
    service::Service,
};
use uuid::Uuid;

use crate::{
    blaze::{
        models::{
            user_sessions::{IpPairAddress, NetData},
            PlayerState,
        },
        pk::{codec::Encodable, packet::Packet, tag::TdfType, types::TdfMap, writer::TdfWriter},
        session::{PushExt, SessionLink, SetGameMessage},
    },
    database::entity::User,
};

pub mod manager;

pub type GameID = u32;

pub struct Game {
    /// Unique ID for this game
    pub id: GameID,
    /// The current game state
    pub state: u8,
    /// The current game setting
    pub setting: u32,
    /// The game attributes
    pub attributes: AttrMap,
    /// The list of players in this game
    pub players: Vec<Player>,
}

impl Service for Game {
    fn stopping(&mut self) {
        // debug!("Game is stopping (GID: {})", self.id);
        // // Remove the stopping game
        // let services = App::services();
        // let _ = services
        //     .game_manager
        //     .do_send(RemoveGameMessage { game_id: self.id });
    }
}

#[derive(Message)]
pub struct UpdateStateMessage {
    pub state: u8,
}

impl Handler<UpdateStateMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdateStateMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.state = msg.state;
        self.notify_state();
    }
}

#[derive(Message)]
pub struct UpdatePlayerAttr {
    pub attr: AttrMap,
    pub pid: u32,
}

impl Handler<UpdatePlayerAttr> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdatePlayerAttr,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(
            4,
            90,
            NotifyPlayerAttr {
                attr: msg.attr.clone(),
                pid: msg.pid,
                gid: self.id,
            },
        );

        let player = self
            .players
            .iter_mut()
            .find(|player| player.user.id == msg.pid);

        if let Some(player) = player {
            player.attr.extend(msg.attr);
        }
    }
}

#[derive(Message)]
pub struct UpdateGameAttrMessage {
    pub attr: AttrMap,
}

impl Handler<UpdateGameAttrMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: UpdateGameAttrMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(
            4,
            80,
            NotifyGameAttr {
                attr: msg.attr.clone(),
                gid: self.id,
            },
        );
        self.attributes.extend(msg.attr);
    }
}

pub struct NotifyPlayerAttr {
    attr: AttrMap,
    pid: u32,
    gid: u32,
}

impl Encodable for NotifyPlayerAttr {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_value(b"ATTR", &self.attr);
        w.tag_u32(b"GID", self.gid);
        w.tag_u32(b"PID", self.pid);
    }
}

pub struct NotifyGameAttr {
    attr: AttrMap,
    gid: u32,
}

impl Encodable for NotifyGameAttr {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_value(b"ATTR", &self.attr);
        w.tag_u32(b"GID", self.gid);
    }
}

impl Game {
    pub fn new(id: u32) -> Link<Game> {
        // TODO: Take attributes provided by client matchmaking
        let this = Self {
            id,
            state: 1,
            setting: 262144,
            attributes: [
                ("coopGameVisibility", "1"),
                ("difficulty", "1"),
                ("difficultyRND", ""),
                ("enemytype", "0"),
                ("enemytypeRND", "1"),
                ("level", "0"),
                ("levelRND", "6"),
                ("missionSlot", "0"),
                ("missiontype", "Custom"),
                ("mode", "contact_multiplayer"),
                ("modifierCount", "0"),
                ("modifiers", ""),
            ]
            .into_iter()
            .collect(),
            players: Vec::with_capacity(4),
        };
        this.start()
    }

    /// Writes the provided packet to all connected sessions.
    /// Does not wait for the write to complete just waits for
    /// it to be placed into each sessions write buffers.
    ///
    /// `packet` The packet to write
    fn push_all(&self, packet: &Packet) {
        self.players
            .iter()
            .for_each(|value| value.link.push(packet.clone()));
    }

    /// Sends a notification packet to all the connected session
    /// with the provided component and contents
    ///
    /// `component` The packet component
    /// `contents`  The packet contents
    fn notify_all<C: Encodable>(&self, component: u16, command: u16, contents: C) {
        let packet = Packet::notify(component, command, contents);
        self.push_all(&packet);
    }

    /// Notifies all players of the current game state
    fn notify_state(&self) {
        self.notify_all(
            4,
            100,
            NotifyStateUpdate {
                game_id: self.id,
                state: self.state,
            },
        );
    }
}

#[derive(Message)]
pub struct GameFinishMessage;

impl Handler<GameFinishMessage> for Game {
    type Response = ();

    fn handle(
        &mut self,
        _msg: GameFinishMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(4, 100, NotifyGameFinish { game_id: self.id })
    }
}

/// Message to add a new player to this game
#[derive(Message)]
pub struct AddPlayerMessage {
    /// The player to add to the game
    pub player: Player,
}

/// Handler for adding a player to the game
impl Handler<AddPlayerMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: AddPlayerMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        let _slot = self.players.len();

        self.players.push(msg.player);

        // Obtain the player that was just added
        let player = self
            .players
            .last()
            .expect("Player was added but is missing from players");
        let packet = Packet::notify(
            4,
            20,
            GameDetails {
                game: self,
                player_id: player.user.id,
            },
        );

        player.link.push(packet);
        player.link.push(Packet::notify(
            4,
            11,
            PostJoinMsg {
                game_id: self.id,
                player_id: player.user.id,
            },
        ));

        // Set current game of this player
        player.set_game(Some(self.id));
    }
}

/// Attributes map type
pub type AttrMap = TdfMap<String, String>;

pub struct Player {
    pub uuid: Uuid,
    pub user: User,
    pub link: SessionLink,
    pub net: NetData,
    pub state: PlayerState,
    pub attr: AttrMap,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.set_game(None);
    }
}

impl Player {
    pub fn new(uuid: Uuid, user: User, link: SessionLink, net: NetData) -> Self {
        Self {
            uuid,
            user,
            link,
            net,
            state: PlayerState::ActiveConnecting,
            attr: AttrMap::default(),
        }
    }

    pub fn set_game(&self, game: Option<GameID>) {
        let _ = self.link.do_send(SetGameMessage { game });
    }

    pub fn encode(&self, game_id: u32, slot: usize, w: &mut TdfWriter) {
        w.tag_empty_blob(b"BLOB");
        w.tag_u64(b"CONG", 1052287650009);
        w.tag_u8(b"CSID", 0);
        w.tag_u8(b"DSUI", 0);
        w.tag_empty_blob(b"EXBL");
        w.tag_u32(b"EXID", self.user.id);
        w.tag_u32(b"GID", game_id);
        w.tag_u8(b"JFPS", 1);
        w.tag_u8(b"JVMM", 1);
        w.tag_u32(b"LOC", 0x64654445);
        w.tag_str(b"NAME", &self.user.username);
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_u32(b"PID", self.user.id);
        IpPairAddress::tag(self.net.addr.as_ref(), b"PNET", w);

        w.tag_u8(b"PSET", 1);
        w.tag_u8(b"RCRE", 0);
        w.tag_str_empty(b"ROLE");
        w.tag_usize(b"SID", slot);
        w.tag_u8(b"SLOT", 0);
        w.tag_value(b"STAT", &self.state);
        w.tag_u16(b"TIDX", 0);
        w.tag_u8(b"TIME", 0); /* Unix timestamp in millseconds */
        w.tag_triple(b"UGID", (30722, 2, 1052287650009u64));
        w.tag_u32(b"UID", self.user.id);
        w.tag_str(b"UUID", &self.uuid.to_string());
        w.tag_group_end();
    }
}

pub struct GameDetails<'a> {
    pub game: &'a Game,
    pub player_id: u32,
}

fn write_admin_list(writer: &mut TdfWriter, game: &Game) {
    writer.tag_list_start(b"ADMN", TdfType::VarInt, game.players.len());
    for player in &game.players {
        writer.write_u32(player.user.id);
    }
}

impl Encodable for GameDetails<'_> {
    fn encode(&self, w: &mut TdfWriter) {
        let game = self.game;
        let host_player = match game.players.first() {
            Some(value) => value,
            None => return,
        };

        // Game details
        w.group(b"GAME", |w| {
            write_admin_list(w, game);
            w.tag_u8(b"APRS", 1);
            w.tag_value(b"ATTR", &game.attributes);
            w.tag_slice_list(b"CAP", &[4, 0, 0, 0]);
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

            w.tag_u64(b"GPVH", 3788120962);
            w.tag_u32(b"GSET", game.setting);
            w.tag_u32(b"GSID", game.id); // SHOULD MATCH START MISSION RESPONSE ID
            w.tag_value(b"GSTA", &game.state);

            w.tag_str_empty(b"GTYP");
            w.tag_str_empty(b"GURL");
            {
                w.tag_list_start(b"HNET", TdfType::Group, 1);
                w.write_byte(2);
                if let Some(addr) = &host_player.net.addr {
                    addr.encode(w);
                }
            }

            w.tag_u8(b"MCAP", 1);
            w.tag_u8(b"MNCP", 1);
            w.tag_str_empty(b"NPSI");
            w.group(b"NQOS", |w| {
                w.tag_u32(b"BWHR", 0);
                w.tag_u32(b"DBPS", 24000000);
                w.tag_u32(b"NAHR", 0);
                w.tag_u32(b"NATT", 0);
                w.tag_u32(b"UBPS", 8000000);
            });

            w.tag_zero(b"NRES");
            w.tag_zero(b"NTOP");
            w.tag_str_empty(b"PGID");
            w.tag_empty_blob(b"PGSR");

            w.group(b"PHST", |w| {
                w.tag_u64(b"CONG", 1052279530202);
                w.tag_u32(b"CSID", 0);
                w.tag_u32(b"HPID", host_player.user.id);
                w.tag_zero(b"HSLT");
            });
            w.tag_u8(b"PRES", 0x1);
            w.tag_u8(b"PRTO", 0);
            w.tag_str(b"PSAS", "bio-syd");
            w.tag_u8(b"PSEU", 0);
            w.tag_u8(b"QCAP", 0);
            w.group(b"RNFO", |w| {
                w.tag_map_start(b"CRIT", TdfType::String, TdfType::Group, 1);
                w.write_empty_str();
                w.tag_u8(b"RCAP", 1);
                w.tag_group_end();
            });
            w.tag_str_empty(b"SCID");
            w.tag_u32(b"SEED", 131492528);
            w.tag_str_empty(b"STMN");

            w.group(b"THST", |w| {
                w.tag_u64(b"CONG", 1052279530202);
                w.tag_u8(b"CSID", 0x0);
                w.tag_u32(b"HPID", host_player.user.id);
                w.tag_u32(b"HSES", host_player.user.id);
                w.tag_u8(b"HSLT", 0x0);
            });

            w.tag_slice_list(b"TIDS", &[65534]);
            w.tag_str(b"UUID", "32d89cf8-6a83-4282-b0a0-5b7a8449de2e");
            w.tag_u8(b"VOIP", 0);
            w.tag_str(b"VSTR", "60-Future739583");
        });

        w.tag_u8(b"LFPJ", 0);
        w.tag_str(b"MNAM", "coopGameVisibility");

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

        w.tag_union_start(b"REAS", 0x3);
        w.group(b"MMSC", |writer| {
            const FIT: u16 = 20000;

            writer.tag_u16(b"FIT", FIT);
            writer.tag_u16(b"FIT", 0);
            writer.tag_u16(b"MAXF", FIT);
            writer.tag_u32(b"MSCD", self.player_id);
            writer.tag_u32(b"MSID", self.player_id);
            writer.tag_u16(b"RSLT", 0);
            writer.tag_u32(b"TOUT", 15000000);
            writer.tag_u32(b"TTM", 51109);
            // TODO: Matchmaking result
            // SUCCESS_CREATED_GAME = 0
            // SUCCESS_JOINED_NEW_GAME = 1
            // SUCCESS_JOINED_EXISTING_GAME = 2
            // SESSION_TIMED_OUT = 3
            // SESSION_CANCELED = 4
            // SESSION_TERMINATED = 5
            // SESSION_ERROR_GAME_SETUP_FAILED = 6
            writer.tag_u32(b"USID", self.player_id);
        });
    }
}

pub struct PostJoinMsg {
    pub player_id: u32,
    pub game_id: u32,
}

impl Encodable for PostJoinMsg {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"CONV", |w| {
            w.tag_zero(b"FCNT");
            w.tag_zero(b"NTOP");
            w.tag_zero(b"TIER");
        });
        w.tag_u8(b"DISP", 1);
        w.tag_u32(b"GID", self.game_id);
        w.tag_triple(b"GRID", (0, 0, 0));
        w.tag_u32(b"MSCD", self.player_id);
        w.tag_u32(b"MSID", self.player_id);
        w.tag_u32(b"QSVR", 0);
        w.tag_u32(b"USID", self.player_id);
    }
}

struct NotifyStateUpdate {
    state: u8,
    game_id: u32,
}

impl Encodable for NotifyStateUpdate {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_u32(b"GID", self.game_id);
        w.tag_u8(b"GSTA", self.state)
    }
}
struct NotifyGameFinish {
    game_id: u32,
}

impl Encodable for NotifyGameFinish {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_u32(b"GID", self.game_id);
        w.tag_u32(b"GRID", self.game_id)
    }
}
