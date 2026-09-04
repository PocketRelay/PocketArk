use pocket_ark::run;

#[allow(unused)]
mod blaze;

mod config;
mod database;
pub mod database_v2;
mod definitions;
mod http;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    run();
}
