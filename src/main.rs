use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::*;
use log4rs::encode::json::JsonEncoder;
use log4rs::init_config;

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

    let (kafka_status_consumer, kafka_status_handle) = kafka::init_status_consumer();
    let status_channel = kafka_status_consumer.clone();

    let (kafka_aap_simulering_consumer, kafka_aap_simulering_handle) = kafka::init_aap_simulering_consumer();
    let aap_simulering_channel = kafka_aap_simulering_consumer.clone();

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::from(status_channel.clone()))
            .app_data(Data::from(aap_simulering_channel.clone()))
            .service(routes::abetal)
            .service(routes::health)
    })
    .bind(&host)?
    .run()
    .await;

    kafka_status_consumer.disable();
    kafka_status_handle.await.unwrap();

    kafka_aap_simulering_consumer.disable();
    kafka_aap_simulering_handle.await.unwrap();

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

