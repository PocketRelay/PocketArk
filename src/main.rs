use pocket_ark::run;

pub mod blaze;

pub mod config;
pub mod http;
pub mod services;
pub mod utils;

#[tokio::main]
async fn main() {
    run().await;
}
