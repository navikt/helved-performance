use actix_web::web::get;
use actix_web::{App, HttpResponse, HttpServer};
use dto::*;
use log::{info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::{init_config, Config};

mod client;
mod dto;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    let host = env_or_default("BIND_ADDRESS", "127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/health", get().to(is_alive))
            .route("/iverksett", get().to(iverksett))
    })
    .bind(&host)?
    .run().await
}

pub fn env_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

async fn is_alive() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn iverksett() -> HttpResponse {
    let mut iverksetting = Iverksetting::new();
    let mut vedtak = Vedtaksdetaljer::new();
    let utbetaling = Utbetaling::new();
    vedtak.add_utbetaling(utbetaling);
    iverksetting.set_vedtak(vedtak);

    info!("iverksetter: {:?} ..", iverksetting);

    let url = "http://utsjekk-oppdrag/api/iverksetting/v2";
    match client::post(url, &iverksetting).await {
        Ok(res) => {
            info!("response: {:?}", res);
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            warn!("response: {:?}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}

fn init_logger() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::helved-performance", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .expect("Failed to build log config");
    init_config(config).expect("Failed to init logger");
}
