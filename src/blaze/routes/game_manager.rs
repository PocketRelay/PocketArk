use crate::{
    blaze::{
        models::game_manager::{
            LeaveGameRequest, MatchmakeRequest, MatchmakeType, MatchmakingResponse,
            ReplayGameRequest, UpdateAttrRequest, UpdateGameAttrRequest, UpdateStateRequest,
        },
        router::Blaze,
        session::{self, GetPlayerMessage, GetUserMessage, SessionLink},
    },
    services::game::{
        manager::{CreateMessage, GetGameMessage},
        NotifyGameReplayMessage, RemovePlayerMessage, UpdateGameAttrMessage, UpdatePlayerAttr,
        UpdateStateMessage,
    },
    state::App,
};

pub async fn create_game(
    session: SessionLink,
    Blaze(req): Blaze<MatchmakeRequest>,
) -> Blaze<MatchmakingResponse> {
    let services = App::services();
    let player = session
        .send(GetPlayerMessage)
        .await
        .expect("Failed to get player");

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
            // TODO: Handle create vs matchmaking

            let _game = services
                .games
                .send(CreateMessage {
                    host: player,
                    attributes: req
                        .attributes
                        .into_iter()
                        .map(|(key, value)| (key, value.value))
                        .collect(),
                })
                .await
                .expect("Failed to create");
        }
    }

    Blaze(MatchmakingResponse { user_id })
}

pub async fn update_game_attr(Blaze(req): Blaze<UpdateGameAttrRequest>) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateGameAttrMessage { attr: req.attr }).await;
}

pub async fn update_player_attr(Blaze(req): Blaze<UpdateAttrRequest>) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game
        .send(UpdatePlayerAttr {
            attr: req.attr,
            pid: req.pid,
        })
        .await;
}

pub async fn update_game_state(Blaze(req): Blaze<UpdateStateRequest>) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateStateMessage { state: req.state }).await;
}

pub async fn replay_game(Blaze(req): Blaze<ReplayGameRequest>) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateStateMessage { state: 130 }).await;
    let _ = game.send(NotifyGameReplayMessage).await;
}

pub async fn leave_game(session: SessionLink, Blaze(req): Blaze<LeaveGameRequest>) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let user = session
        .send(GetUserMessage)
        .await
        .expect("Failed to get user");
    let _ = game
        .send(RemovePlayerMessage {
            user_id: user.id,
            reason: req.reas,
        })
        .await;
}
