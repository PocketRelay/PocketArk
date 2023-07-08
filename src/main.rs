use std::net::SocketAddr;

use futures::StreamExt;
use packet::PacketCodec;
use tokio::{
    io::split,
    net::{TcpListener, TcpStream},
};
use tokio_native_tls::TlsStream;
use tokio_util::codec::{FramedRead, FramedWrite};

mod blaze;
mod http;
mod packet;
mod structs;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

async fn start_server() {
    let addr: SocketAddr = "0.0.0.0:10853".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    let acceptor = native_tls::TlsAcceptor::builder(
        native_tls::Identity::from_pkcs12(include_bytes!("identity.p12"), "password").unwrap(),
    )
    .build()
    .unwrap();
    let acceptor = tokio_native_tls::TlsAcceptor::from(acceptor);

    while let Ok((stream, _addr)) = listener.accept().await {
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            let mut stream = match acceptor.accept(stream).await {
                Ok(value) => value,
                Err(err) => {
                    return;
                }
            };

            handle_client(stream).await;
        });
    }
}

async fn handle_client(mut stream: TlsStream<TcpStream>) {
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
    }
}
