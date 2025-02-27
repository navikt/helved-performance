use log4rs::append::console::ConsoleAppender;
use log4rs::config::*;
use log4rs::encode::json::JsonEncoder;
use log4rs::init_config;
use server::init_server;

mod client;
mod dto;
mod job;
mod kafka;
mod server;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    init_server().await
}

pub fn env_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

fn init_logger() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::helved-performance", log::LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(log::LevelFilter::Debug))
        .expect("Failed to build log config");

    init_config(config).expect("Failed to init logger");
}

