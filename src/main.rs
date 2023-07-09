use log::LevelFilter;
use tokio::signal;

mod blaze;
mod http;
mod utils;

#[tokio::main]
async fn main() {
    utils::logging::setup(LevelFilter::Debug);

    tokio::spawn(http::start_server());
    tokio::spawn(blaze::start_server());

    let _ = signal::ctrl_c().await;
}
