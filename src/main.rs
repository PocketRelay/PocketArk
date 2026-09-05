use pocket_ark::run;

#[allow(unused)]
mod blaze;

mod config;
pub mod database;
mod definitions;
mod http;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    run().await;
}
