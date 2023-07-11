use std::{collections::HashMap, sync::atomic::AtomicU32};

use interlink::prelude::Link;
use tokio::sync::RwLock;

use crate::services::Game;

static GAME_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Default)]
pub struct GameManager(RwLock<HashMap<u32, Link<Game>>>);

pub async fn create_game() {}
