use std::collections::HashMap;
use std::sync::{Arc };
use tokio::sync::Mutex;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::*;
use log4rs::encode::json::JsonEncoder;
use log4rs::init_config;

use crate::routes::{StatusPubSub, SimPubSub};

mod models;
mod kafka;
mod routes;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    init_server().await
}

pub async fn init_server() -> anyhow::Result<()> {
    let host = env_or_default("BIND_ADDRESS", "127.0.0.1:8080");

    let status_pending: StatusPubSub = Arc::new(Mutex::new(HashMap::new()));
    actix_web::rt::spawn(kafka::status_consumer(status_pending.clone()));

    let simulering_pending: SimPubSub = Arc::new(Mutex::new(HashMap::new()));
    actix_web::rt::spawn(kafka::dryrun_consumer(simulering_pending.clone()));

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(status_pending.clone()))
            .app_data(Data::new(simulering_pending.clone()))
            .service(routes::abetal_dp)
            .service(routes::abetal_dp_tx)
            .service(routes::abetal_aap)
            .service(routes::abetal_ts)
            .service(routes::abetal_tp)
            .service(routes::abetal_historisk)
            .service(routes::health)
    })
    .bind(&host)?
    .run()
    .await;

    Ok(())
}

pub fn env_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_owned(),
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

