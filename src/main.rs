use pocket_ark::run;

#[allow(unused)]
pub mod blaze;

pub mod config;
pub mod database;
pub mod definitions;
pub mod http;
pub mod services;
pub mod utils;

#[tokio::main]
async fn main() {
    run().await;
}
