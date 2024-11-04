use std::sync::Mutex;

use actix_web::http::StatusCode;
use actix_web::web::{get, Data};
use actix_web::{App, HttpResponse, HttpResponseBuilder, HttpServer};
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

    let state = Data::new(AppState {
        state: Mutex::new(State::Stopped),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", get().to(is_alive))
            .route("/iverksett", get().to(iverksett))
            .route("/start", get().to(start))
            .route("/stop", get().to(stop))
            .route("/debug", get().to(debug))
    })
    .bind(&host)?
    .run()
    .await
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

struct AppState {
    state: Mutex<State>,
}

enum State {
    Started,
    Stopped,
}

async fn debug(data: Data<AppState>) -> HttpResponse {
    match *data.state.lock().unwrap() {
        State::Started => HttpResponse::Ok().body("Started"),
        State::Stopped => HttpResponse::Ok().body("Stopped"),
    }
}
async fn start(data: Data<AppState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = State::Started;
    HttpResponse::Ok().body("Started")
}

async fn stop(data: Data<AppState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = State::Stopped;
    HttpResponse::Ok().body("Stopped")
}

async fn iverksett() -> HttpResponse {
    let mut iverksetting = Iverksetting::new();
    let mut vedtak = Vedtaksdetaljer::new();
    let utbetaling = Utbetaling::new();
    vedtak.add_utbetaling(utbetaling);
    iverksetting.set_vedtak(vedtak);

    info!("iverksetter: {:?} ..", iverksetting);

    let url = "http://utsjekk/api/iverksetting/v2";

    match client::post(url, &iverksetting).await {
        Ok(res) => {
            info!("response: {:?}", &res);
            let status = &res.status().as_u16();
            let status = StatusCode::from_u16(status.to_owned()).expect("Invalid status code");
            let body = res.text().await.unwrap();
            HttpResponseBuilder::new(status).body(body)
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
