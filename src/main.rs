use axum::Extension;
use definitions::i18n::I18n;
use definitions::strike_teams::StrikeTeams;
use definitions::{
    badges::Badges, challenges::Challenges, classes::Classes, items::Items,
    level_tables::LevelTables, match_modifiers::MatchModifiers,
};
use log::LevelFilter;
use log::error;
use services::mission::MissionBackgroundTask;
use services::sessions::Sessions;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::join;
use tokio::signal;
use utils::{constants::SERVER_PORT, signing::SigningKey};

use crate::services::game::store::Games;
use crate::services::tunnel::TunnelService;

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
    utils::logging::setup(LevelFilter::Debug);

    // Pre-initialize all shared definitions
    _ = Items::get();
    _ = Classes::get();
    _ = LevelTables::get();
    _ = Challenges::get();
    _ = I18n::get();
    _ = Badges::get();
    _ = MatchModifiers::get();
    _ = StrikeTeams::get();

    let (db, signing_key) = join!(crate::database::init(), SigningKey::global());

    // Start the strike team mission background task
    MissionBackgroundTask::new(db.clone()).start();

    let games = Arc::new(Games::default());
    let sessions = Arc::new(Sessions::new(signing_key));
    let (tunnel_service, _udp_forward_rx) = TunnelService::new();
    let tunnel_service = Arc::new(tunnel_service);

    let mut router = blaze::routes::router();
    router.add_extension(db.clone());
    router.add_extension(games.clone());
    router.add_extension(tunnel_service.clone());
    router.add_extension(sessions.clone());
    let router = router.build();

    let router = http::routes::router()
        .layer(Extension(router))
        .layer(Extension(db))
        .layer(Extension(games))
        .layer(Extension(tunnel_service))
        .layer(Extension(sessions));

    let addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, SERVER_PORT));
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to bind HTTP server on {}: {:?}", addr, err);
            return;
        }
    };

    if let Err(err) = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        _ = signal::ctrl_c().await;
    })
    .await
    {
        error!("Error while running server: {:?}", err);
    }
}
