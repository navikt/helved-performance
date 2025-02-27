use actix_web::web::{self, get, post, Data};
use actix_web::{App, HttpResponse, HttpServer};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::{client, kafka};
use crate::job::{JobState, State};
use crate::{env_or_default, job::init_job};

pub async fn init_server() -> anyhow::Result<()> {
    let host = env_or_default("BIND_ADDRESS", "127.0.0.1:8080");

    let (job_state, job_handle) = init_job();
    info!("jobs state {:?}", &job_state);

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::from(job_state.clone()))
            .route("/health", get().to(health))
            .route("/start", get().to(start))
            .route("/stop", get().to(stop))
            .route("/sleep", post().to(sleep))
            .route("/debug", get().to(debug))
            .route("/iverksett", get().to(iverksett))
            .route("/abetal", post().to(abetal))
    })
    .bind(&host)?
    .run()
    .await;

    job_handle.await.unwrap();

    Ok(())
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn start(data: Data<JobState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = State::Started;
    info!("started");
    HttpResponse::Ok().body("started")
}

async fn stop(data: Data<JobState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = State::Stopped;
    info!("stopped");
    HttpResponse::Ok().body("stopped")
}

#[derive(Deserialize)]
struct Sleep {
    ms: u64,
}

async fn sleep(sleep: web::Json<Sleep> , data: Data<JobState>) -> HttpResponse {
    let mut sleep_ms = data.sleep_ms.lock().unwrap();
    *sleep_ms = sleep.ms;
    info!("sleep between jobs: {} ms", sleep.ms);
    HttpResponse::Ok().body(format!("sleep between jobs: {} ms", sleep.ms))
}

async fn debug(data: Data<JobState>) -> HttpResponse {
    let debug = format!(
        "state: {:?} sleep_ms: {:?}",
        data.state.lock().unwrap(),
        data.sleep_ms.lock().unwrap(),
    );
    HttpResponse::Ok().body(debug)
    // match *data.state.lock().unwrap() {
    //     State::Started => HttpResponse::Ok().body("started"),
    //     State::Stopped => HttpResponse::Ok().body("stopped"),
    // }
}

async fn iverksett(_: Data<JobState>) -> HttpResponse {
    client::iverksett().await
}

async fn abetal(json: web::Json<kafka::Utbetaling>) -> HttpResponse {
    let uid = Uuid::new_v4();
    kafka::abetal(uid, json.0).await;
    HttpResponse::Ok().body(uid.to_string())
}

