use crate::state::App;
use crate::utils::constants::SERVER_PORT;
use axum::Extension;
use log::error;
use log::LevelFilter;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::signal;

#[allow(unused)]
mod blaze;

mod database;
mod http;
mod services;
mod utils;

mod state;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "tower_http=trace");

    utils::logging::setup(LevelFilter::Debug);

    App::init().await;

    let database = crate::database::init().await;

    let addr: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), SERVER_PORT));

    let mut router = blaze::routes::router();
    router.add_extension(database.clone());

    let router = router.build();

    let router = http::routes::router()
        .layer(Extension(router))
        .layer(Extension(database));

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
