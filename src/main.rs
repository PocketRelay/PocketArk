use crate::utils::constants::SERVER_PORT;
use crate::utils::signing::SigningKey;
use axum::Extension;
use log::error;
use log::LevelFilter;
use services::challenges::ChallengeDefinitions;
use services::character::class::ClassDefinitions;
use services::character::levels::LevelTables;
use services::game::manager::GameManager;
use services::i18n::I18n;
use services::items::ItemDefinitions;
use services::match_data::MatchDataDefinitions;
use services::sessions::Sessions;
use services::strike_teams::StrikeTeamDefinitions;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::join;
use tokio::signal;

#[allow(unused)]
mod blaze;

mod database;
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
    _ = ItemDefinitions::get();
    _ = ClassDefinitions::get();
    _ = LevelTables::get();
    _ = ChallengeDefinitions::get();
    _ = I18n::get();
    _ = MatchDataDefinitions::get();
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
