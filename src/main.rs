use log::LevelFilter;
use tokio::{select, signal};

mod blaze;
mod http;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    utils::logging::setup(LevelFilter::Debug);

    select! {
        _ = http::start_server() => {

        }

        _ = signal::ctrl_c() => {

        }
    }
}
