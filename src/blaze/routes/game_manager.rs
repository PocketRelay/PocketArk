use crate::{
    blaze::{
        models::game_manager::CreateGameResp,
        pk::{codec::Decodable, error::DecodeResult, reader::TdfReader},
        session::{GetPlayerMessage, Session, SessionLink},
    },
    services::game::{
        manager::{CreateMessage, GetGameMessage},
        AttrMap, UpdateGameAttrMessage, UpdatePlayerAttr, UpdateStateMessage,
    },
    state::App,
};

pub async fn create_game(session: &mut SessionLink) -> CreateGameResp {
    let services = App::services();
    let player = session
        .send(GetPlayerMessage)
        .await
        .expect("Failed to get player");

    let game = services
        .games
        .send(CreateMessage { host: player })
        .await
        .expect("Failed to create");
    CreateGameResp
}

pub struct UpdateGameAttrRequest {
    attr: AttrMap,
    gid: u32,
}
impl Decodable for UpdateGameAttrRequest {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let attr = r.tag(b"ATTR")?;
        let gid = r.tag(b"GID")?;
        Ok(Self { attr, gid })
    }
}

pub async fn update_game_attr(session: &mut SessionLink, req: UpdateGameAttrRequest) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateGameAttrMessage { attr: req.attr }).await;
}

pub struct UpdateAttrRequest {
    attr: AttrMap,
    gid: u32,
    pid: u32,
}

impl Decodable for UpdateAttrRequest {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let attr = r.tag(b"ATTR")?;
        let gid = r.tag(b"GID")?;
        let pid = r.tag(b"PID")?;
        Ok(Self { attr, gid, pid })
    }
}

pub async fn update_player_attr(session: &mut SessionLink, req: UpdateAttrRequest) {
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

pub struct UpdateStateRequest {
    gid: u32,
    state: u8,
}
impl Decodable for UpdateStateRequest {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let gid = r.tag(b"GID")?;
        let state = r.tag(b"GSTA")?;
        Ok(Self { gid, state })
    }
}

pub async fn update_game_state(session: &mut SessionLink, req: UpdateStateRequest) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateStateMessage { state: req.state }).await;
}

pub struct ReplyGameRequest {
    gid: u32,
}
impl Decodable for ReplyGameRequest {
    fn decode(r: &mut TdfReader) -> DecodeResult<Self> {
        let gid = r.tag(b"GID")?;
        Ok(Self { gid })
    }
}

pub async fn replay_game(session: &mut SessionLink, req: ReplyGameRequest) {
    let services = App::services();
    let game = services
        .games
        .send(GetGameMessage { game_id: req.gid })
        .await
        .expect("Failed to create")
        .expect("Unknown game");
    let _ = game.send(UpdateStateMessage { state: 130 }).await;
}
