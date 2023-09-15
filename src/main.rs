use crate::state::App;
use axum::Extension;
use log::LevelFilter;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::{select, signal};

use crate::utils::constants::SERVER_PORT;
use axum_server::tls_openssl::OpenSSLConfig;
use log::error;
use openssl::{
    pkey::PKey,
    rsa::Rsa,
    ssl::{SslAcceptor, SslMethod},
    x509::X509,
};

#[allow(unused)]
mod blaze;

mod database;
mod http;
mod services;
mod utils;

mod state;

const CERTIFICATE: &[u8] = include_bytes!("./resources/identity/cert.der");
const PRIVATE_KEY: &[u8] = include_bytes!("./resources/identity/key.pem");

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

    let ssl_config = ssl_config();
    let server_future =
        axum_server::bind_openssl(addr, ssl_config).serve(router.into_make_service());
    let close_future = signal::ctrl_c();

    select! {
        result = server_future => {
            if let Err(err) = result {
                error!("Failed to bind HTTP server on {}: {:?}", addr, err);
                panic!();
            }
        }
           // Handle the server being stopped with CTRL+C
        _ = close_future => {}
    }
}

fn ssl_config() -> OpenSSLConfig {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();

    let crt = X509::from_der(CERTIFICATE).expect("Server certificate is invalid");
    let pkey = PKey::from_rsa(
        Rsa::private_key_from_pem(PRIVATE_KEY).expect("Server private key is invalid"),
    )
    .expect("Server private key is invalid");

    acceptor
        .set_certificate(&crt)
        .expect("Failed to set HTTP server certificate");
    acceptor
        .set_private_key(&pkey)
        .expect("Failed to set HTTP server private key");
    OpenSSLConfig::try_from(acceptor).expect("Failed to create OpenSSL config")
}
