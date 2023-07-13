use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    process::exit,
};

use axum_server::tls_openssl::OpenSSLConfig;
use log::error;
use openssl::{
    pkey::PKey,
    rsa::Rsa,
    ssl::{SslAcceptor, SslMethod},
    x509::X509,
};

use crate::utils::constants::SERVER_PORT;

pub mod middleware;
pub mod models;
mod routes;

const CERTIFICATE: &[u8] = include_bytes!("../resources/identity/cert.der");
const PRIVATE_KEY: &[u8] = include_bytes!("../resources/identity/key.pem");

pub async fn start_server() {
    let addr: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), SERVER_PORT));

    let router = routes::router();
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

    let config = OpenSSLConfig::try_from(acceptor).expect("Failed to create OpenSSL config");

    if let Err(err) = axum_server::bind_openssl(addr, config)
        .serve(router.into_make_service())
        .await
    {
        error!("Failed to start HTTP server: {}", err);
        exit(1);
    }
}
