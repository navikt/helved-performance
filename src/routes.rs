use std::time::{Duration, Instant};
use actix_web::rt::time::sleep;
use actix_web::web::{self, Data};
use actix_web::{get, post, HttpResponse};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::kafka;
use crate::utsjekk;
use crate::job;

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/iverksett")]
pub async fn iverksett(_: Data<job::JobState>) -> HttpResponse {
    utsjekk::iverksett().await
}

#[post("/abetal/{uid}")]
pub async fn abetal(
    data: Data<kafka::Channel>,
    uid: web::Path<Uuid>,
    json: web::Json<kafka::Utbetaling>,
) -> HttpResponse {
    let uid = uid.into_inner();
    {
        let tx = &data.uid.lock().unwrap().0;
        tx.send(uid).unwrap();
    }

    kafka::abetal(uid, json.0).await; 

    let start = Instant::now();
    let mut last_status: Option<kafka::StatusReply> = None;
    loop {
        if start.elapsed() >= Duration::from_secs(50) { // simulering kan være treg
            break;
        }
        {
            let rx = &data.status.lock().unwrap().1;
            if let Ok(rec) = rx.recv_timeout(Duration::from_millis(10)) {
                last_status = Some(rec.clone());
                match rec.status {
                    kafka::Status::Mottatt    => {},
                    kafka::Status::HosOppdrag => {},
                    kafka::Status::Feilet     => break,
                    kafka::Status::Ok         => break,
                }
            }
        }
        sleep(Duration::from_millis(1)).await;
    }
    match last_status {
        Some(status) => HttpResponse::Ok().json(status),
        None => HttpResponse::Accepted().finish(),
    }
}

#[derive(Deserialize)]
struct Sleep {
    ms: u64,
}

#[get("/job/start")]
pub async fn job_start(data: Data<job::JobState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = job::State::Started;
    info!("started");
    HttpResponse::Ok().body("started")
}

#[get("/job/stop")]
pub async fn job_stop(data: Data<job::JobState>) -> HttpResponse {
    let mut state = data.state.lock().unwrap();
    *state = job::State::Stopped;
    info!("stopped");
    HttpResponse::Ok().body("stopped")
}

#[post("/job/sleep")]
pub async fn job_sleep(sleep: web::Json<Sleep> , data: Data<job::JobState>) -> HttpResponse {
    let mut sleep_ms = data.sleep_ms.lock().unwrap();
    *sleep_ms = sleep.ms;
    info!("sleep between jobs: {} ms", sleep.ms);
    HttpResponse::Ok().body(format!("sleep between jobs: {} ms", sleep.ms))
}

#[get("/job/debug")]
pub async fn job_debug(data: Data<job::JobState>) -> HttpResponse {
    let debug = format!(
        "state: {:?} sleep_ms: {:?}",
        data.state.lock().unwrap(),
        data.sleep_ms.lock().unwrap(),
    );
    HttpResponse::Ok().body(debug)
}

