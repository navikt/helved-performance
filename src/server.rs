use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use actix_web::rt::spawn;
use actix_web::web::{self, get, post, Data};
use actix_web::{post, App, HttpResponse, HttpServer};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::kafka::StatusReply;
use crate::{client, kafka};
use crate::job::{JobState, State};
use crate::{env_or_default, job::init_job};

pub async fn init_server() -> anyhow::Result<()> {
    let host = env_or_default("BIND_ADDRESS", "127.0.0.1:8080");

    let (job_state, job_handle) = init_job();
    info!("jobs state {:?}", &job_state);

    let channel = Arc::new(Channel::default());
    let handle = spawn(kafka::status_listener(channel.clone()));
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::from(channel.clone()))
            .app_data(Data::from(job_state.clone()))
            .route("/health", get().to(health))
            .route("/start", get().to(start))
            .route("/stop", get().to(stop))
            .route("/sleep", post().to(sleep))
            .route("/debug", get().to(debug))
            .route("/iverksett", get().to(iverksett))
            .service(abetal)
    })
    .bind(&host)?
    .run()
    .await;

    job_handle.await.unwrap();
    handle.await.unwrap();

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

#[post("/abetal/{uid}")]
async fn abetal(
    data: Data<Channel>,
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
    let mut last_status: Option<StatusReply> = None;
    loop {
        if start.elapsed() >= Duration::from_secs(30) {
            return match last_status {
                Some(status) => HttpResponse::Ok().json(status),
                None => HttpResponse::Accepted().finish(),
            };
        }

        let rx = &data.status.lock().unwrap().1;
        if let Ok(rec) = rx.recv_timeout(Duration::from_secs(1)) {
            match rec.status {
                kafka::Status::Ok | kafka::Status::Feilet => return HttpResponse::Ok().json(rec),
                _ => last_status = Some(rec),
            }
        }
    }
}

pub struct Channel {
    pub status: Mutex<(Sender<StatusReply>, Receiver<StatusReply>)>,
    pub uid: Mutex<(Sender<Uuid>, Receiver<Uuid>)>,
}

impl Default for Channel {
    fn default() -> Self {
        Channel {
            status: Mutex::new(mpsc::channel()),
            uid: Mutex::new(mpsc::channel()),
        }
    }
}
