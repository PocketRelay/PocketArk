use crate::{
    blaze::{
        models::game_manager::{
            LeaveGameRequest, MatchmakeRequest, MatchmakeType, MatchmakingResponse,
            ReplayGameRequest, UpdateAttrRequest, UpdateGameAttrRequest, UpdateStateRequest,
        },
        router::{Blaze, SessionAuth},
        session::{self, SessionLink},
    },
    services::game::Player,
    state::App,
};

pub async fn create_game(
    session: SessionLink,
    player: Player,
    Blaze(req): Blaze<MatchmakeRequest>,
) -> Blaze<MatchmakingResponse> {
    let services = App::services();

    let user_id = player.user.id;

    match req.ty {
        MatchmakeType::QuickMatch => {

            // TODO:
            // - Add to matchmaking queue
            // - Send async matchmaking update (4, 12)
            // - Couldn't find one? create new one
            // - found one? send game details
        }
        MatchmakeType::CreatePublicGame => {
            let attributes = req
                .attributes
                .into_iter()
                .map(|(key, value)| (key, value.value))
                .collect();
            tokio::spawn(async move {
                let (_link, _id) = services.games.create(player, attributes).await;
            });
        }
    }

    Blaze(MatchmakingResponse { user_id })
}

pub async fn update_game_attr(Blaze(req): Blaze<UpdateGameAttrRequest>) {
    let services = App::services();
    let game = services
        .games
        .get_game(req.gid)
        .await
        .expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_attributes(req.attr);
}

pub async fn update_player_attr(Blaze(req): Blaze<UpdateAttrRequest>) {
    let services = App::services();
    let game = services
        .games
        .get_game(req.gid)
        .await
        .expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_player_attributes(req.pid, req.attr);
}

pub async fn update_game_state(Blaze(req): Blaze<UpdateStateRequest>) {
    let services = App::services();

    let game = services
        .games
        .get_game(req.gid)
        .await
        .expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_state(req.state);
}

pub async fn replay_game(Blaze(req): Blaze<ReplayGameRequest>) {
    let services = App::services();

    let game = services
        .games
        .get_game(req.gid)
        .await
        .expect("Unknown game");

    let game = &mut *game.write().await;
    game.set_state(130);
    game.notify_game_replay();
}

pub async fn leave_game(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Blaze(req): Blaze<LeaveGameRequest>,
) {
    let services = App::services();

    let game = services
        .games
        .get_game(req.gid)
        .await
        .expect("Unknown game");

    let game = &mut *game.write().await;
    game.remove_player(user.id, req.reas);
}
