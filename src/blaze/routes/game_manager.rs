use log::{debug, info};
use tdf::TdfMap;

use crate::{
    blaze::{
        models::game_manager::{
            GameSetupContext, LeaveGameRequest, MatchmakeScenario, MatchmakingResult, PlayerState,
            ReplayGameRequest, StartMatchmakingScenarioRequest, StartMatchmakingScenarioResponse,
            UpdateAttrRequest, UpdateGameAttrRequest, UpdateStateRequest,
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
use std::sync::Arc;

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

    let attributes: TdfMap<String, String> = req
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
    game.set_state(130);
    game.notify_game_replay();
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
