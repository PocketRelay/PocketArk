use axum::Extension;
use definitions::i18n::I18n;
use definitions::strike_teams::StrikeTeams;
use definitions::{
    badges::Badges, challenges::Challenges, classes::Classes, items::Items,
    level_tables::LevelTables, match_modifiers::MatchModifiers,
};
use log::error;
use log::info;
use services::mission::MissionBackgroundTask;
use services::sessions::Sessions;
use tokio::net::TcpListener;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::join;
use tokio::signal;
use utils::signing::SigningKey;

use crate::config::{TunnelConfig, VERSION, load_config};
use crate::services::game::store::Games;
use crate::services::tunnel::udp_tunnel::start_udp_tunnel;
use crate::services::tunnel::{TunnelService, tunnel_keep_alive};
use crate::utils::logging;

#[allow(unused)]
mod blaze;

mod config;
mod database;
mod definitions;
mod http;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    // Load configuration
    let mut config = load_config().unwrap_or_default();

    // Initialize logging
    logging::setup(config.logging);

    // Create the server socket address while the port is still available
    let addr: SocketAddr = SocketAddr::new(config.host, config.port);

    // This step may take longer than expected so its spawned instead of joined
    tokio::spawn(logging::log_connection_urls(config.port));

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

    let (tunnel_service, udp_forward_rx) = TunnelService::new();
    let tunnel_service = Arc::new(tunnel_service);

    // Start tunnel if not disabled
    if !matches!(config.tunnel, TunnelConfig::Disabled) {
        tokio::spawn(tunnel_keep_alive(tunnel_service.clone()));

        // Start UDP tunnel if enabled
        if config.udp_tunnel.enabled {
            // Create tunnel server socket address
            let tunnel_addr: SocketAddr = SocketAddr::new(config.host, config.udp_tunnel.port);

            // Start the tunnel service server
            if let Err(err) = start_udp_tunnel(
                tunnel_addr,
                tunnel_service.clone(),
                sessions.clone(),
                udp_forward_rx,
            )
            .await
            {
                error!("failed to start UDP tunnel server: {err}");

                // Disable failed UDP tunnel
                config.udp_tunnel.enabled = false;
            }
        }
    }

    let config = Arc::new(config);

    // Initialize session router
    let router = blaze::routes::router()
        .extension(db.clone())
        .extension(config.clone())
        .extension(games.clone())
        .extension(tunnel_service.clone())
        .extension(sessions.clone())
        .build();

    // Create the HTTP router
    let router = http::routes::router()
        .layer(Extension(router))
        .layer(Extension(db))
        .layer(Extension(games))
        .layer(Extension(tunnel_service))
        .layer(Extension(sessions))
        .into_make_service_with_connect_info::<SocketAddr>();

    info!("Starting server on {addr} (v{VERSION})");

    // Start the TCP listener
    let listener = match TcpListener::bind(addr).await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to bind HTTP server pm {addr}: {err:?}");
            return;
        }
    };

    if let Err(err) = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            _ = signal::ctrl_c().await;
        })
        .await
    {
        error!("Error within HTTP server {err:?}");
    }
}
