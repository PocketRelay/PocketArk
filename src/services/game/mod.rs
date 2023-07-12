use interlink::{prelude::Link, service::Service};
use uuid::Uuid;

use crate::blaze::{
    models::{
        user_sessions::{IpPairAddress, NetData},
        PlayerState,
    },
    pk::{codec::Encodable, tag::TdfType, types::TdfMap, writer::TdfWriter},
    session::{SessionLink, User},
};

pub mod manager;

pub struct Game {
    /// Unique ID for this game
    pub id: u32,
    /// The current game state
    pub state: u8,
    /// The current game setting
    pub setting: u16,
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

impl Game {
    pub fn new(id: u32) -> Link<Game> {
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
        w.tag_str(b"NAME", &self.user.name);
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
            w.tag_u16(b"GSET", game.setting);
            w.tag_u64(b"GSID", 60474918);
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
                w.tag_u16(b"BWHR", 0);
                w.tag_u16(b"DBPS", 24000000);
                w.tag_u16(b"NAHR", 0);
                w.tag_u16(b"NATT", 0);
                w.tag_u16(b"UBPS", 8000000);
            });

            w.tag_zero(b"NRES");
            w.tag_zero(b"NTOP");
            w.tag_str_empty(b"PGID");
            w.tag_empty_blob(b"PGSR");

            w.group(b"PHST", |w| {
                w.tag_u32(b"CONG", 1052279530202);
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
