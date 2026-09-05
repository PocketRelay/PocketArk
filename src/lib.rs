#![recursion_limit = "256"]
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, middleware};
use bytes::Bytes;
use definitions::i18n::I18n;
use definitions::strike_teams::StrikeTeams;
use definitions::{
    badges::Badges, challenges::Challenges, classes::Classes, items::Items,
    level_tables::LevelTables, match_modifiers::MatchModifiers,
};
use hyper::{StatusCode, Uri};
use log::error;
use log::info;
use services::mission::MissionBackgroundTask;
use services::sessions::Sessions;
use tokio::net::TcpListener;

use http_body_util::BodyExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::join;
use tokio::signal;
use utils::signing::SigningKey;

use crate::config::{TunnelConfig, VERSION, load_config};
use crate::definitions::packs::Packs;
use crate::services::game::matchmaking::Matchmaking;
use crate::services::game::store::Games;
use crate::services::tunnel::udp_tunnel::start_udp_tunnel;
use crate::services::tunnel::{TunnelService, tunnel_keep_alive};
use crate::utils::logging;

#[allow(unused)]
pub mod blaze;

pub mod config;
mod database;
pub mod database_v2;
pub mod definitions;
pub(crate) mod http;
pub mod services;
pub mod utils;

pub async fn run() {
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
    _ = Packs::get();

    let (db, signing_key) = join!(crate::database::init(), SigningKey::global());

    // Start the strike team mission background task
    MissionBackgroundTask::new(db.clone()).start();

    let games = Arc::new(Games::default());
    let sessions = Arc::new(Sessions::new(signing_key));
    let matchmaking = Arc::new(Matchmaking::default());

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
        .extension(matchmaking.clone())
        .build();

    // Create the HTTP router
    let router = http::routes::router()
        .layer(Extension(router))
        .layer(Extension(db))
        .layer(Extension(games))
        .layer(Extension(tunnel_service))
        .layer(Extension(sessions))
        .layer(Extension(config))
        .layer(middleware::from_fn(print_request_response))
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

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let uri = req.uri().clone();

    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", &uri, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", &uri, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(
    direction: &str,
    uri: &Uri,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        log::debug!("{direction} uri = {uri}, body = {body}");
    }

    Ok(bytes)
}
