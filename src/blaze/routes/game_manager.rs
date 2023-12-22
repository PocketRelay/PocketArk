use crate::{
    blaze::{
        models::{
            game_manager::{
                GameSetupContext, LeaveGameRequest, MatchmakeScenario, MatchmakingResult,
                ReplayGameRequest, StartMatchmakingScenarioRequest,
                StartMatchmakingScenarioResponse, UpdateAttrRequest, UpdateGameAttrRequest,
                UpdateStateRequest,
            },
            PlayerState,
        },
        router::{Blaze, Extension, SessionAuth},
        session::{self, SessionLink},
    },
    services::{
        game::{self, Player, DEFAULT_FIT},
        game_manager::GameManager,
    },
};
use std::sync::Arc;

pub async fn start_matchmaking_scenario(
    session: SessionLink,
    mut player: Player,
    Blaze(req): Blaze<StartMatchmakingScenarioRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) -> Blaze<StartMatchmakingScenarioResponse> {
    let user_id = player.user.id;

    match req.ty {
        MatchmakeScenario::QuickMatch => {
            // TODO:
            // - Add to matchmaking queue
            // - Send async matchmaking update (4, 12)
            // - Couldn't find one? create new one
            // - found one? send game details
        }
        MatchmakeScenario::CreatePublicGame => {
            let attributes = req
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

            // Player is the host player (They are connected by default)
            player.state = PlayerState::ActiveConnected;

            // Create the new game
            let (game_ref, game_id) = game_manager.create(attributes).await;

            // Add the player to the game
            game_manager
                .add_to_game(
                    game_ref,
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
                )
                .await;
        }
    }

    Blaze(StartMatchmakingScenarioResponse { user_id })
}

pub async fn update_game_attr(
    Blaze(req): Blaze<UpdateGameAttrRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) {
    let game = game_manager.get_game(req.gid).await.expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_attributes(req.attr);
}

pub async fn update_player_attr(
    Blaze(req): Blaze<UpdateAttrRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) {
    let game = game_manager.get_game(req.gid).await.expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_player_attributes(req.pid, req.attr);
}

pub async fn update_game_state(
    Blaze(req): Blaze<UpdateStateRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) {
    let game = game_manager.get_game(req.gid).await.expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_state(req.state);
}

pub async fn replay_game(
    Blaze(req): Blaze<ReplayGameRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) {
    let game = game_manager.get_game(req.gid).await.expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_state(130);
    game.notify_game_replay();
}

pub async fn leave_game(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Blaze(req): Blaze<LeaveGameRequest>,
    Extension(game_manager): Extension<Arc<GameManager>>,
) {
    let game = game_manager.get_game(req.gid).await.expect("Unknown game");

    let game = &mut *game.write().await;
    game.remove_player(user.id, req.reas);
}
