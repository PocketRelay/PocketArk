use log::{debug, info};
use tdf::TdfMap;

use crate::{
    blaze::{
        models::{
            errors::ServerResult,
            game_manager::{
                AddAdminPlayerRequest, GameManagerError, GameSetupContext, GameState,
                LeaveGameRequest, MatchmakeScenario, MatchmakingResult,
                MeshEndpointsConnectedRequest, PlayerNetConnectionStatus, PlayerState,
                ReplayGameRequest, SetSettingRequest, StartMatchmakingScenarioRequest,
                StartMatchmakingScenarioResponse, UpdateAttrRequest, UpdateGameAttrRequest,
                UpdateStateRequest,
            },
        },
        router::{Blaze, Extension, SessionAuth},
        session::{self, SessionLink},
    },
    config::Config,
    services::{
        activity::AttributeError,
        game::{
            self, DEFAULT_FIT, Game, GameAddPlayerExt, GameJoinableState, matchmaking::Matchmaking,
            player::GamePlayer, rules::RuleSet, store::Games,
        },
        tunnel::TunnelService,
    },
};
use std::{sync::Arc, thread::sleep, time::Duration};

pub async fn start_matchmaking_scenario(
    session: SessionLink,
    mut player: GamePlayer,
    Blaze(req): Blaze<StartMatchmakingScenarioRequest>,
    Extension(games): Extension<Arc<Games>>,
    Extension(config): Extension<Arc<Config>>,
    Extension(tunnel_service): Extension<Arc<TunnelService>>,
    Extension(matchmaking): Extension<Arc<Matchmaking>>,
) -> Blaze<StartMatchmakingScenarioResponse> {
    let user_id = player.user.id;

    let mut attributes: TdfMap<String, String> = req
        .attributes
        .into_iter()
        .filter_map(|(key, value)| {
            let inner = value.inner?;
            let value = match inner.value {
                tdf::TdfGenericValue::String(value) => value,
                _ => return None,
            };
            Some((key, value))
        })
        .collect();

    match req.ty {
        MatchmakeScenario::QuickMatch => {
            // TODO:
            // - Add to matchmaking queue
            // - Send async matchmaking update (4, 12)
            // - Couldn't find one? create new one
            // - found one? send game details

            let rules = RuleSet::new(attributes.into_inner());

            info!("Player {} started matchmaking", player.user.username);

            // Find a game thats currently joinable and matches the required rules
            match games.get_by_rule_set(&rules) {
                Some((game_id, game_ref)) => {
                    debug!("Found matching game (GID: {game_id})");

                    // Add the player to the game
                    matchmaking.add_from_matchmaking(&tunnel_service, &config, game_ref, player);
                }
                None => {
                    matchmaking.queue(player, rules);
                }
            };
        }
        MatchmakeScenario::CreatePublicGame => {
            // Player is the host player (They are connected by default)
            player.state = PlayerState::ActiveConnected;

            attributes.insert("isInLobby".to_string(), "true".to_string());
            attributes.insert("lockState".to_string(), "0".to_string());
            attributes.insert("mode".to_string(), "contact_multiplayer".to_string());
            if let Some(difficulty) = attributes.get("difficulty") {
                attributes.insert("difficultyUI".to_string(), difficulty.clone());
            }
            if let Some(ty) = attributes.get("enemytype") {
                attributes.insert("enemytypeUI".to_string(), ty.clone());
            }
            if let Some(level) = attributes.get("level") {
                attributes.insert("levelUI".to_string(), level.clone());
            }

            let game_id = games.next_id();
            let game = Game::new(game_id, attributes, games.clone(), tunnel_service.clone());
            let game_ref = games.insert(game);

            // Add the player to the game
            game_ref.add_player(
                &tunnel_service,
                &config,
                player,
                session,
                GameSetupContext::Matchmaking {
                    fit_score: DEFAULT_FIT,
                    fit_score_2: 0,
                    max_fit_score: DEFAULT_FIT,
                    id_1: user_id,
                    id_2: user_id,
                    result: MatchmakingResult::CreatedGame,
                    tout: 15000000,
                    ttm: 51109,
                    id_3: user_id,
                },
            );

            matchmaking.process_queue(&tunnel_service, &config, &game_ref, game_id);
        }
    }

    Blaze(StartMatchmakingScenarioResponse { user_id })
}

pub async fn cancel_matchmaking_scenario(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Extension(matchmaking): Extension<Arc<Matchmaking>>,
) -> Blaze<()> {
    info!("Player {} cancelled matchmaking", user.username);
    let user_id = user.id;
    session.data.clear_game();
    matchmaking.remove(user_id);
    Blaze(())
}

pub async fn update_game_attr(
    Blaze(req): Blaze<UpdateGameAttrRequest>,
    Extension(games): Extension<Arc<Games>>,
    Extension(config): Extension<Arc<Config>>,
    Extension(tunnel_service): Extension<Arc<TunnelService>>,
    Extension(matchmaking): Extension<Arc<Matchmaking>>,
) {
    let game_id = req.gid;
    let game_ref = games.get_by_id(game_id).expect("Unknown game");

    // Update matchmaking for the changed game
    let join_state = {
        let game = &mut *game_ref.write();
        game.set_attributes(req.attr);

        game.joinable_state(None)
    };

    if let GameJoinableState::Joinable = join_state {
        matchmaking.process_queue(&tunnel_service, &config, &game_ref, game_id);
    }
}

pub async fn update_player_attr(
    Blaze(req): Blaze<UpdateAttrRequest>,
    Extension(games): Extension<Arc<Games>>,
) {
    let game = games.get_by_id(req.gid).expect("Unknown game");
    let game = &mut *game.write();
    game.set_player_attributes(req.pid, req.attr);
}

pub async fn update_game_state(
    Blaze(req): Blaze<UpdateStateRequest>,
    Extension(games): Extension<Arc<Games>>,
) {
    let game = games.get_by_id(req.gid).expect("Unknown game");

    let game = &mut *game.write();
    game.set_state(req.state);
}

pub async fn replay_game(
    Blaze(req): Blaze<ReplayGameRequest>,
    Extension(games): Extension<Arc<Games>>,
) {
    let game = games.get_by_id(req.gid).expect("Unknown game");
    let game = &mut *game.write();
    game.replay();
}

pub async fn leave_game(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Blaze(req): Blaze<LeaveGameRequest>,
    Extension(games): Extension<Arc<Games>>,
) {
    let game = games.get_by_id(req.gid).expect("Unknown game");

    let game = &mut *game.write();
    game.remove_player(user.id, req.reas);
}

pub async fn finalize_game_creation(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Extension(games): Extension<Arc<Games>>,
) {
    // Should make the game be available for joining
}

pub async fn mesh_endpoints_connected(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Extension(games): Extension<Arc<Games>>,
    Blaze(MeshEndpointsConnectedRequest {
        game_id,
        target_group_id,
        ..
    }): Blaze<MeshEndpointsConnectedRequest>,
) -> ServerResult<()> {
    let game_ref = games
        .get_by_id(game_id)
        .ok_or(GameManagerError::InvalidGameId)?;

    let game = &mut *game_ref.write();

    // Ensure the host is the one making the change
    if game.is_host_player(user.id) {
        let target_id = target_group_id.id;
        game.update_mesh(target_id as u32, PlayerNetConnectionStatus::Connected);
    }

    Ok(())
}

pub async fn add_admin_player(
    SessionAuth(player): SessionAuth,
    Extension(games): Extension<Arc<Games>>,
    Blaze(AddAdminPlayerRequest { game_id, player_id }): Blaze<AddAdminPlayerRequest>,
) -> ServerResult<()> {
    let link = games
        .get_by_id(game_id)
        .ok_or(GameManagerError::InvalidGameId)?;

    let game = &mut *link.write();

    // Ensure the host is the one making the change
    if game.is_host_player(player.id) {
        game.add_admin_player(player_id);
    }

    Ok(())
}

pub async fn set_setting(
    Extension(games): Extension<Arc<Games>>,
    Blaze(SetSettingRequest { game_id, setting }): Blaze<SetSettingRequest>,
) -> ServerResult<()> {
    let game_ref = games
        .get_by_id(game_id)
        .ok_or(GameManagerError::InvalidGameId)?;

    game_ref.write().set_settings(setting);

    Ok(())
}
