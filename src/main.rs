use log::LevelFilter;
use tokio::{select, signal};

use crate::{database::entity::User, state::App};

#[allow(unused)]
mod blaze;

mod database;
mod http;
mod services;
mod utils;

mod state;

#[tokio::main]
async fn main() {
    utils::logging::setup(LevelFilter::Debug);

    App::init().await;

    select! {
        _ = http::start_server() => {

        }

        _ = signal::ctrl_c() => {

        }
    }
}
