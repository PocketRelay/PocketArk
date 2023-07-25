use interlink::prelude::*;
use std::collections::HashMap;

use crate::blaze::models::PlayerState;

use super::{AddPlayerMessage, Game, GameID, Player};

/// Manager which controls all the active games on the server
/// commanding them to do different actions and removing them
/// once they are no longer used
#[derive(Service)]
pub struct GameManager {
    /// The map of games to the actual game address
    games: HashMap<GameID, Link<Game>>,
    /// Stored value for the ID to give the next game
    next_id: GameID,
}

impl GameManager {
    /// Starts a new game manager service returning its link
    pub fn start() -> Link<GameManager> {
        let this = GameManager {
            games: Default::default(),
            next_id: 1,
        };
        this.start()
    }
}

/// Message for creating a new game using the game manager
/// responds with a link to the created game and its ID
#[derive(Message)]
#[msg(rtype = "(Link<Game>, GameID)")]
pub struct CreateMessage {
    /// The host player for the game
    pub host: Player,
}

/// Handler for creating games
impl Handler<CreateMessage> for GameManager {
    type Response = Mr<CreateMessage>;

    fn handle(
        &mut self,
        mut msg: CreateMessage,
        _ctx: &mut ServiceContext<Self>,
    ) -> Self::Response {
        let id = self.next_id;

        self.next_id = self.next_id.wrapping_add(1);

        msg.host.state = PlayerState::ActiveConnected;

        let link = Game::new(id);
        self.games.insert(id, link.clone());

        let _ = link.do_send(AddPlayerMessage { player: msg.host });

        Mr((link, id))
    }
}

/// Message for requesting a link to a game with the provided
/// ID responds with a link to the game if it exists
#[derive(Message)]
#[msg(rtype = "Option<Link<Game>>")]
pub struct GetGameMessage {
    /// The ID of the game to get a link to
    pub game_id: GameID,
}

/// Handler for getting a specific game
impl Handler<GetGameMessage> for GameManager {
    type Response = Mr<GetGameMessage>;

    fn handle(&mut self, msg: GetGameMessage, _ctx: &mut ServiceContext<Self>) -> Self::Response {
        let link = self.games.get(&msg.game_id).cloned();
        Mr(link)
    }
}

// /// Message for attempting to add a player to any existing
// /// games within this game manager
// #[derive(Message)]
// #[msg(rtype = "TryAddResult")]
// pub struct TryAddMessage {
//     /// The player to attempt to add
//     pub player: Player,
//     // The set of rules the player requires the game has
//     // pub rule_set: Arc<RuleSet>,
// }

// /// Result of attempting to add a player. Success will
// /// consume the game player and Failure will return the
// /// game player back
// pub enum TryAddResult {
//     /// The player was added to the game
//     Success,
//     /// The player failed to be added and was returned back
//     Failure(Player),
// }

// /// Handler for attempting to add a player
// impl Handler<TryAddMessage> for GameManager {
//     type Response = Fr<TryAddMessage>;

//     fn handle(&mut self, msg: TryAddMessage, _ctx: &mut ServiceContext<Self>) -> Self::Response {
//         // Take a copy of the current games list
//         let games = self.games.clone();

//         Fr::new(Box::pin(async move {
//             let player = msg.player;

//             // Message asking for the game joinable state
//             let msg = CheckJoinableMessage {
//                 rule_set: Some(msg.rule_set),
//             };

//             // Attempt to find a game thats joinable
//             for (id, link) in games {
//                 // Check if the game is joinable
//                 if let Ok(GameJoinableState::Joinable) = link.send(msg.clone()).await {
//                     debug!("Found matching game (GID: {})", id);
//                     let msid = player.player.id;
//                     let _ = link.do_send(AddPlayerMessage {
//                         player,
//                         context: GameSetupContext::Matchmaking(msid),
//                     });
//                     return TryAddResult::Success;
//                 }
//             }

//             TryAddResult::Failure(player)
//         }))
//     }
// }

/// Message for removing a game from the manager
#[derive(Message)]
pub struct RemoveGameMessage {
    /// The ID of the game to remove
    pub game_id: GameID,
}

/// Handler for removing a game
impl Handler<RemoveGameMessage> for GameManager {
    type Response = ();

    fn handle(&mut self, msg: RemoveGameMessage, _ctx: &mut ServiceContext<Self>) {
        // Remove the game
        if let Some(value) = self.games.remove(&msg.game_id) {
            value.stop();
        }
    }
}
