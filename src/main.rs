use actix_web::{App, HttpResponse, HttpServer};
use actix_web::web::get;
use dto::*;

mod dto;
mod client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = env_or_default("BIND_ADDRESS", "127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/health", get().to(is_alive))
            .route("/iverksett", get().to(iverksett))
    })
        .bind(host)?
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

async fn iverksett() -> HttpResponse {
    let mut iverksetting = Iverksetting::new();
    let mut vedtak = Vedtaksdetaljer::new();
    let utbetaling = Utbetaling::new();
    vedtak.add_utbetaling(utbetaling);
    iverksetting.set_vedtak(vedtak);

    let url = "http://utsjekk-oppdrag/api/iverksetting/v2";
    match client::post(url, &iverksetting).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

