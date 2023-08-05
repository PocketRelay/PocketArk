use crate::{
    blaze::{
        models::game_manager::{
            CreateGameResp, LeaveGameRequest, MatchmakeRequest, ReplayGameRequest,
            UpdateAttrRequest, UpdateGameAttrRequest, UpdateStateRequest,
        },
        session::{self, GetPlayerMessage, GetUserMessage, SessionLink},
    },
    services::game::{
        manager::{CreateMessage, GetGameMessage},
        NotifyGameReplayMessage, RemovePlayerMessage, UpdateGameAttrMessage, UpdatePlayerAttr,
        UpdateStateMessage,
    },
    state::App,
};

pub async fn create_game(session: &mut SessionLink, req: MatchmakeRequest) -> CreateGameResp {
    let services = App::services();
    let player = session
        .send(GetPlayerMessage)
        .await
        .expect("Failed to get player");

    // TODO: Handle create vs matchmaking

    let _game = services
        .games
        .send(CreateMessage { host: player })
        .await
        .expect("Failed to create");
    CreateGameResp
}

pub async fn update_game_attr(_session: &mut SessionLink, req: UpdateGameAttrRequest) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateGameAttrMessage { attr: req.attr }).await;
}

pub async fn update_player_attr(_session: &mut SessionLink, req: UpdateAttrRequest) {
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

pub async fn update_game_state(_session: &mut SessionLink, req: UpdateStateRequest) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateStateMessage { state: req.state }).await;
}

pub async fn replay_game(_session: &mut SessionLink, req: ReplayGameRequest) {
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

pub async fn leave_game(session: &mut SessionLink, req: LeaveGameRequest) {
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
