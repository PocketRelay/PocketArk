use axum::Extension;
use definitions::i18n::I18n;
use definitions::{
    badges::Badges, challenges::Challenges, classes::Classes, items::Items,
    level_tables::LevelTables, match_modifiers::MatchModifiers, striketeams::StrikeTeamDefinitions,
};
use log::error;
use log::LevelFilter;
use services::{game_manager::GameManager, sessions::Sessions};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::join;
use tokio::signal;
use utils::{constants::SERVER_PORT, signing::SigningKey};

#[allow(unused)]
mod blaze;

mod database;
mod definitions;
mod http;
mod services;
mod utils;

/// The server version extracted from the Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "tower_http=trace");

    utils::logging::setup(LevelFilter::Debug);

    // Pre-initialize all shared definitions
    _ = Items::get();
    _ = Classes::get();
    _ = LevelTables::get();
    _ = Challenges::get();
    _ = I18n::get();
    _ = Badges::get();
    _ = MatchModifiers::get();
    _ = StrikeTeamDefinitions::get();

    let (database, signing_key) = join!(crate::database::init(), SigningKey::global());

    let game_manager = Arc::new(GameManager::new());
    let sessions = Arc::new(Sessions::new(signing_key));

    let mut router = blaze::routes::router();
    router.add_extension(database.clone());
    router.add_extension(game_manager.clone());
    let router = router.build();

    let router = http::routes::router()
        .layer(Extension(router))
        .layer(Extension(database))
        .layer(Extension(game_manager))
        .layer(Extension(sessions));

    let addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, SERVER_PORT));
    if let Err(err) = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            _ = signal::ctrl_c().await;
        })
        .await
    {
        error!("Failed to bind HTTP server on {}: {:?}", addr, err);
    }
}
