use std::{collections::HashMap, future::ready};

use chrono::Utc;
use interlink::{
    prelude::{Fr, Handler, Link, Message, Mr, Sfr},
    service::Service,
};
use log::error;
use sea_orm::{DatabaseConnection, DbErr};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    blaze::{
        components,
        models::{
            user_sessions::{IpPairAddress, NetData},
            PlayerState,
        },
        pk::{codec::Encodable, packet::Packet, tag::TdfType, types::TdfMap, writer::TdfWriter},
        session::{PushExt, SessionLink, SetGameMessage},
    },
    database::entity::{
        challenge_progress::ProgressUpdateType, ChallengeProgress, Character, Currency, SharedData,
        User,
    },
    http::models::{
        auth::Sku,
        character::{LevelTable, Xp},
        mission::{
            ChallengeStatusChange, ChallengeUpdate, CompleteMissionData, MissionDetails,
            MissionModifier, MissionPlayerData, MissionPlayerInfo, PlayerInfoBadge,
            PlayerInfoResult, PrestigeData, PrestigeProgression, RewardSource,
        },
    },
    state::App,
};

use super::match_data::MatchDataService;

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

    pub modifiers: Vec<MissionModifier>,
    pub mission_data: Option<CompleteMissionData>,
    pub processed_data: Option<MissionDetails>,
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
#[msg(rtype = "Option<MissionDetails>")]
pub struct GetMissionDataMessage;

impl Handler<GetMissionDataMessage> for Game {
    type Response = Sfr<Self, GetMissionDataMessage>;
    fn handle(
        &mut self,
        msg: GetMissionDataMessage,
        ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        Sfr::new(move |act: &mut Game, ctx| {
            Box::pin(async move {
                if let Some(processed) = act.processed_data.clone() {
                    return Some(processed);
                }

                let mission_data = act.mission_data.clone()?;

                let now = Utc::now();
                let db = App::database();

                let waves = mission_data
                    .player_data
                    .iter()
                    .map(|value| value.waves_completed)
                    .max()
                    .unwrap_or_default();

                let level: String = mission_data
                    .modifiers
                    .iter()
                    .find(|value| value.name == "level")
                    .map(|value| value.value.clone())
                    .unwrap_or_else(|| "MPAqua".to_string());
                let difficulty: String = mission_data
                    .modifiers
                    .iter()
                    .find(|value| value.name == "difficulty")
                    .map(|value| value.value.clone())
                    .unwrap_or_else(|| "bronze".to_string());
                let enemy_type: String = mission_data
                    .modifiers
                    .iter()
                    .find(|value| value.name == "enemytype")
                    .map(|value| value.value.clone())
                    .unwrap_or_else(|| "outlaw".to_string());

                let mut player_infos = Vec::with_capacity(mission_data.player_data.len());

                for value in &mission_data.player_data {
                    match process_player_data(db, value, &mission_data).await {
                        Ok(info) => {
                            player_infos.push(info);
                        }
                        Err(err) => {
                            error!("Error while processing player: {}", err);
                        }
                    }
                }

                let data = MissionDetails {
                    sku: Sku::default(),
                    name: mission_data.match_id,
                    duration_sec: mission_data.duration_sec,
                    percent_complete: mission_data.percent_complete,
                    waves_encountered: waves,
                    extraction_state: mission_data.extraction_state,
                    enemy_type,
                    difficulty,
                    map: level,
                    start: now,
                    end: now,
                    processed: now,
                    player_infos,
                    modifiers: mission_data.modifiers,
                };

                act.processed_data = Some(data.clone());

                Some(data)
            })
        })
    }
}

#[derive(Debug, Error)]
enum PlayerDataProcessError {
    #[error("Unknown user")]
    UnknownUser,
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error("Missing character")]
    MissingCharacter,
    #[error("Missing class")]
    MissingClass,
}

pub struct PlayerDataBuilder {
    pub score: u32,
    pub xp_earned: u32,
    pub reward_sources: Vec<RewardSource>,
    pub total_currency: HashMap<String, u32>,
    pub prestige_progression: PrestigeProgression,
    pub items_earned: Vec<Value>,
    pub challenges_updates: HashMap<String, ChallengeUpdate>,
    pub badges: Vec<PlayerInfoBadge>,
}

impl PlayerDataBuilder {
    pub fn new() -> Self {
        Self {
            score: 0,
            xp_earned: 0,
            reward_sources: Vec::new(),
            total_currency: HashMap::new(),
            prestige_progression: PrestigeProgression::default(),
            items_earned: Vec::new(),
            challenges_updates: HashMap::new(),
            badges: Vec::new(),
        }
    }

    fn append_prestige(map: &mut HashMap<Uuid, PrestigeData>, shared_data: &SharedData) {
        // Insert the before change
        shared_data.shared_progression.0.iter().for_each(|value| {
            map.insert(
                value.name,
                PrestigeData {
                    level: value.level,
                    name: value.name,
                    xp: value.xp.current,
                },
            );
        });
    }

    pub fn append_prestige_before(&mut self, shared_data: &SharedData) {
        Self::append_prestige(&mut self.prestige_progression.before, shared_data)
    }
    pub fn append_prestige_after(&mut self, shared_data: &SharedData) {
        Self::append_prestige(&mut self.prestige_progression.after, shared_data)
    }

    pub fn add_reward_xp(&mut self, name: &str, xp: u32) {
        // Ignore adding nothing
        if xp == 0 {
            return;
        }

        // Append earned xp
        self.xp_earned += xp;

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            existing.xp += xp;
        } else {
            self.reward_sources.push(RewardSource {
                currencies: HashMap::new(),
                xp,
                name: name.to_string(),
            });
        }
    }

    pub fn add_reward_currency(&mut self, name: &str, currency: &str, value: u32) {
        // Ignore adding nothing
        if value == 0 {
            return;
        }

        // Append currencies to total currrency

        if let Some(existing) = self.total_currency.get_mut(currency) {
            *existing += value
        } else {
            self.total_currency.insert(currency.to_string(), value);
        }

        if let Some(existing) = self
            .reward_sources
            .iter_mut()
            .find(|value| value.name.eq(name))
        {
            // Update currency within reward

            if let Some(existing) = existing.currencies.get_mut(currency) {
                *existing = existing.saturating_add(value);
            } else {
                existing.currencies.insert(currency.to_string(), value);
            }
        } else {
            let mut currencies = HashMap::new();
            currencies.insert(currency.to_string(), value);

            self.reward_sources.push(RewardSource {
                currencies,
                xp: 0,
                name: name.to_string(),
            });
        }
    }
}

async fn process_player_data(
    db: &'static DatabaseConnection,
    data: &MissionPlayerData,
    mission_data: &CompleteMissionData,
) -> Result<MissionPlayerInfo, PlayerDataProcessError> {
    let services = App::services();
    let user = User::get_user(db, data.nucleus_id)
        .await?
        .ok_or(PlayerDataProcessError::UnknownUser)?;
    let mut shared_data = SharedData::get_from_user(&user, db).await?;

    let character = Character::find_by_id_user(db, &user, shared_data.active_character_id)
        .await?
        .ok_or(PlayerDataProcessError::MissingCharacter)?;

    let class = services
        .defs
        .classes
        .list()
        .iter()
        .find(|value| value.name.eq(&character.class_name))
        .ok_or(PlayerDataProcessError::MissingClass)?;

    let mut data_builder = PlayerDataBuilder::new();

    // Tally up initial base scores awarded from all activities
    data_builder.score = data
        .activity_report
        .activities
        .iter()
        .map(|value| value.attributes.score)
        .sum();

    // Update changed challenges
    for (index, challenge_update) in data
        .activity_report
        .activities
        .iter()
        .filter_map(|activity| services.activity.process_activity(activity))
        .enumerate()
    {
        let (model, update_counter, status_change) =
            ChallengeProgress::handle_update(db, &user, challenge_update).await?;
        let status_change = match status_change {
            ProgressUpdateType::Changed => ChallengeStatusChange::Changed,
            ProgressUpdateType::Created => ChallengeStatusChange::Notify,
        };

        data_builder.challenges_updates.insert(
            (index + 1).to_string(),
            ChallengeUpdate {
                challenge_id: model.challenge_id,
                counters: vec![update_counter],
                status_change,
            },
        );
    }

    // Gives awards and badges for each activity
    data.activity_report
        .activities
        .iter()
        .filter_map(|activity| services.match_data.get_by_activity(activity))
        .for_each(|(badge, progress, levels)| {
            let badge_name = badge.name.to_string();
            let mut xp_reward: u32 = 0;
            let mut currency_reward = 0;
            let mut level_names = Vec::with_capacity(levels.len());

            let level_name = levels.last().map(|value| value.name.to_string());
            levels.into_iter().for_each(|badge_level| {
                xp_reward += badge_level.xp_reward;
                currency_reward += badge_level.currency_reward;
                level_names.push(badge_level.name.clone());
            });

            data_builder.add_reward_xp(&badge_name, xp_reward);
            data_builder.add_reward_currency(&badge_name, &badge.currency, currency_reward);

            data_builder.badges.push(PlayerInfoBadge {
                count: progress,
                level_name,
                rewarded_levels: level_names,
                name: badge.name,
            })
        });

    // Compute modifier amounts
    compute_modifiers(
        &services.match_data,
        &mission_data.modifiers,
        &mut data_builder,
    );

    let level_tables = &services.defs.level_tables;

    // Character leveling
    let level_table = level_tables
        .lookup(&class.level_name)
        .expect("Missing class level table");

    let previous_xp = character.xp;
    let previous_level = character.level;

    let (new_xp, level) = compute_leveling(
        level_table,
        character.xp,
        character.level,
        data_builder.xp_earned,
    );

    // Insert the before change
    data_builder.append_prestige_before(&shared_data);

    // Character prestige leveling
    {
        let level_table = level_tables
            .lookup(&class.prestige_level_name)
            .expect("Missing prestige level table");

        let prestige_value = shared_data
            .shared_progression
            .0
            .iter_mut()
            .find(|value| value.name.eq(&class.prestige_level_name));

        // Update the prestive value in-place
        if let Some(prestige_value) = prestige_value {
            let (new_xp, level) = compute_leveling(
                level_table,
                prestige_value.xp,
                prestige_value.level,
                data_builder.xp_earned,
            );

            prestige_value.xp = new_xp;
            prestige_value.level = level;

            // Save the changed progression
            shared_data = shared_data.save_progression(db).await?;
        }
    }

    // Insert after change
    data_builder.append_prestige_after(&shared_data);

    let character = if new_xp != previous_xp || level > previous_level {
        character.update_xp(db, new_xp, level).await?
    } else {
        character
    };

    let mut total_currencies_earned = Vec::new();
    for (key, value) in data_builder.total_currency {
        let mut currency = Currency::create_or_update(db, &user, key, value).await?;
        currency.balance = value;
        total_currencies_earned.push(currency);
    }

    let result = PlayerInfoResult {
        challenges_updated: data_builder.challenges_updates,
        items_earned: data_builder.items_earned,
        xp_earned: data_builder.xp_earned,
        previous_xp: previous_xp.current,
        current_xp: new_xp.current,
        previous_level,
        level: character.level,
        leveled_up: character.level != previous_level,
        score: data_builder.score,
        total_score: data_builder.score,
        character_class_name: class.name,
        total_currencies_earned,
        reward_sources: data_builder.reward_sources,
        prestige_progression: data_builder.prestige_progression,
    };

    Ok(MissionPlayerInfo {
        activities_processed: true,
        bonuses: vec![],
        activities: vec![],
        badges: data_builder.badges,
        stats: data.stats.clone(),
        result,
        pid: user.id,
        persona_id: user.id,
        persona_display_name: user.username,
        character_id: character.character_id,
        character_class: character.class_name,
        modifiers: vec![],
        session_id: Uuid::new_v4(),
        wave_participation: data.waves_in_match,
        present_at_end: data.present_at_end,
    })
}

/// Computes the xp and currency rewards from the provided match modifiers
/// appending them to the provided data builder
fn compute_modifiers(
    match_data: &MatchDataService,
    modifiers: &[MissionModifier],
    data_builder: &mut PlayerDataBuilder,
) {
    modifiers
        .iter()
        .filter_map(|modifier| match_data.get_modifier_entry(&modifier.name, &modifier.value))
        .for_each(|(modifier, modifier_entry)| {
            if let Some(xp_data) = &modifier_entry.xp_data {
                let amount = xp_data.get_amount(data_builder.xp_earned);
                data_builder.add_reward_xp(&modifier.name, amount);
            }

            modifier_entry
                .currency_data
                .iter()
                .for_each(|(key, modifier_data)| {
                    let amount = modifier_data.get_amount(
                        data_builder
                            .total_currency
                            .get(key)
                            .copied()
                            .unwrap_or_default(),
                    );
                    data_builder.add_reward_currency(&modifier.name, key, amount);
                });
        });
}

/// Computes the new xp and level values from the xp earned in
/// the provided data builder
pub fn compute_leveling(
    level_table: &LevelTable,
    mut xp: Xp,
    mut level: u32,
    xp_earned: u32,
) -> (Xp, u32) {
    xp.current = xp.current.saturating_add(xp_earned);

    while xp.current > xp.next {
        let next_lvl = level_table.get_entry_xp(level + 1);
        if let Some(next_xp) = next_lvl {
            level += 1;

            // Subtract the old next amount from earnings
            xp.current -= xp.next;

            // Assign new next and last values
            xp.last = xp.next;
            xp.next = next_xp;
        }
    }

    (xp, level)
}

#[derive(Message)]
pub struct SetModifiersMessage {
    pub modifiers: Vec<MissionModifier>,
}

impl Handler<SetModifiersMessage> for Game {
    type Response = ();
    fn handle(
        &mut self,
        msg: SetModifiersMessage,
        ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.modifiers = msg.modifiers;
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
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_PLAYER_ATTR_UPDATE,
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
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_GAME_ATTR_UPDATE,
            NotifyGameAttr {
                attr: msg.attr.clone(),
                gid: self.id,
            },
        );
        self.attributes.extend(msg.attr);
    }
}

#[derive(Message)]
pub struct SetCompleteMissionMessage {
    pub mission_data: CompleteMissionData,
}

impl Handler<SetCompleteMissionMessage> for Game {
    type Response = ();

    fn handle(
        &mut self,
        msg: SetCompleteMissionMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.mission_data = Some(msg.mission_data)
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
            modifiers: Vec::new(),
            mission_data: None,
            processed_data: None,
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
            components::game_manager::COMPONENT,
            components::game_manager::NOTIFY_GAME_STATE_UPDATE,
            NotifyStateUpdate {
                game_id: self.id,
                state: self.state,
            },
        );
    }
}

#[derive(Message)]
pub struct NotifyGameReplayMessage;

impl Handler<NotifyGameReplayMessage> for Game {
    type Response = ();

    fn handle(
        &mut self,
        _msg: NotifyGameReplayMessage,
        _ctx: &mut interlink::service::ServiceContext<Self>,
    ) -> Self::Response {
        self.notify_all(4, 113, NotifyGameReplay { game_id: self.id })
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
        w.tag_u32(b"CONG", self.user.id);
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
        w.tag_triple(b"UGID", (30722, 2, self.user.id));
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
                w.tag_u32(b"CONG", host_player.user.id);
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
                w.tag_u32(b"CONG", host_player.user.id);
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
struct NotifyGameReplay {
    game_id: u32,
}

impl Encodable for NotifyGameReplay {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_u32(b"GID", self.game_id);
        w.tag_u32(b"GRID", self.game_id)
    }
}
