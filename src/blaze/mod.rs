use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    process::exit,
};

use futures::StreamExt;
use log::error;
use packet::PacketCodec;
use tokio::{
    io::split,
    net::{TcpListener, TcpStream},
};
use tokio_native_tls::TlsStream;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::utils::constants::TCP_SERVER_PORT;
use tokio_native_tls::{
    native_tls::{Identity, TlsAcceptor},
    TlsAcceptor as TokioTlsAcceptor,
};

mod models;
mod packet;
mod router;
mod routes;

const IDENTITY_CHAIN: &[u8] = include_bytes!("../resources/identity/identity.p12");
const IDENTITY_PASSWORD: &str = "password";

pub async fn start_server() {
    let addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        TCP_SERVER_PORT,
    ));

    let identity: Identity = match Identity::from_pkcs12(IDENTITY_CHAIN, IDENTITY_PASSWORD) {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to create server identity: {}", err);
            exit(1);
        }
    };

    let acceptor: TokioTlsAcceptor =
        TokioTlsAcceptor::from(TlsAcceptor::new(identity).expect("Failed to create TLS acceptor"));

    let listener: TcpListener = match TcpListener::bind(addr).await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to bind TCP listner: {}", err);
            exit(1);
        }
    };

    while let Ok((stream, _addr)) = listener.accept().await {
        let acceptor: TokioTlsAcceptor = acceptor.clone();

        tokio::spawn(async move {
            let stream = match acceptor.accept(stream).await {
                Ok(value) => value,
                Err(err) => {
                    return;
                }
            };

            handle_client(stream).await;
        });
    }
}

async fn handle_client(stream: TlsStream<TcpStream>) {
    let (read, write) = split(stream);
    let mut read = FramedRead::new(read, PacketCodec);
    let mut write = FramedWrite::new(write, PacketCodec);

    while let Some(packet) = read.next().await {
        let packet = match packet {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Failed to read packet: {}", err);
                return;
            }
        };

        println!("Got packet: {:?}", &packet.header)
    }
}
