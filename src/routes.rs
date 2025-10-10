use actix_web::http::StatusCode;
use actix_web::web::{self, Data};
use actix_web::{HttpResponse, HttpResponseBuilder, get, post};
use futures::future::select_all;
use log::info;
use std::sync::mpsc::Receiver;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use uuid::Uuid;

use crate::kafka;
use crate::models;
use crate::models::dryrun::Simulering;
use crate::models::status::{Reply, Status};

pub type PendingMap<T> = Arc<Mutex<HashMap<Uuid, mpsc::Sender<T>>>>;

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/fail")]
pub async fn fail() -> HttpResponse {
    log::error!("some log message for testing purpose");
    HttpResponse::Ok().finish()
}

#[post("/abetal/aap")]
pub async fn abetal_aap(
    status_pending: Data<PendingMap<models::status::Reply>>,
    sim_pending: Data<PendingMap<models::dryrun::Simulering>>,
    json: web::Json<models::aap::Utbetaling>,
) -> HttpResponse {
    let key = Uuid::new_v4();
    info!("abetal {}", &key);

    let (status_tx, status_rx) = mpsc::channel();
    status_pending.lock().unwrap().insert(key, status_tx);

    let sim_rx = if json.0.dryrun {
        let (sim_tx, sim_rx) = mpsc::channel();
        sim_pending.lock().unwrap().insert(key, sim_tx);
        Some(sim_rx)
    } else {
        None
    };

    kafka::produce_utbetaling(key, models::Utbetaling::Aap(&json.0)).await;

    let mut handlers = Vec::new();

    if let Some(sim_rx) = sim_rx {
        handlers.push(actix_web::rt::spawn(simulering_handler(sim_rx)));
    }

    handlers.push(actix_web::rt::spawn(status_handler(status_rx)));

    let (first_done, _idx, rest) = select_all(handlers).await;

    for h in rest {
        h.abort();
    }

    first_done.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
}

#[post("/abetal/dp")]
pub async fn abetal_dp(
    status_pending: Data<PendingMap<models::status::Reply>>,
    sim_pending: Data<PendingMap<models::dryrun::Simulering>>,
    json: web::Json<models::dp::Utbetaling>,
) -> HttpResponse {
    let key = Uuid::new_v4();
    info!("abetal {}", &key);

    let (status_tx, status_rx) = mpsc::channel();
    status_pending.lock().unwrap().insert(key, status_tx);

    let sim_rx = if json.0.dryrun {
        let (sim_tx, sim_rx) = mpsc::channel();
        sim_pending.lock().unwrap().insert(key, sim_tx);
        Some(sim_rx)
    } else {
        None
    };

    kafka::produce_utbetaling(key, models::Utbetaling::Dp(&json.0)).await;

    let mut handlers = Vec::new();

    if let Some(sim_rx) = sim_rx {
        handlers.push(actix_web::rt::spawn(simulering_handler(sim_rx)));
    }

    handlers.push(actix_web::rt::spawn(status_handler(status_rx)));

    let (first_done, _idx, rest) = select_all(handlers).await;

    for h in rest {
        h.abort();
    }

    first_done.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
}

#[post("/abetal/ts")]
pub async fn abetal_ts(
    status_pending: Data<PendingMap<models::status::Reply>>,
    sim_pending: Data<PendingMap<models::dryrun::Simulering>>,
    json: web::Json<models::ts::Utbetaling>,
) -> HttpResponse {
    let key = Uuid::new_v4();
    info!("abetal {}", &key);

    let (status_tx, status_rx) = mpsc::channel();
    status_pending.lock().unwrap().insert(key, status_tx);

    let sim_rx = if json.0.dryrun {
        let (sim_tx, sim_rx) = mpsc::channel();
        sim_pending.lock().unwrap().insert(key, sim_tx);
        Some(sim_rx)
    } else {
        None
    };

    kafka::produce_utbetaling(key, models::Utbetaling::Ts(&json.0)).await;

    let mut handlers = Vec::new();

    if let Some(sim_rx) = sim_rx {
        handlers.push(actix_web::rt::spawn(simulering_handler(sim_rx)));
    }

    handlers.push(actix_web::rt::spawn(status_handler(status_rx)));

    let (first_done, _idx, rest) = select_all(handlers).await;

    for h in rest {
        h.abort();
    }

    first_done.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
}

#[post("/abetal/tp")]
pub async fn abetal_tp(
    status_pending: Data<PendingMap<models::status::Reply>>,
    sim_pending: Data<PendingMap<models::dryrun::Simulering>>,
    json: web::Json<models::tp::Utbetaling>,
) -> HttpResponse {
    let key = Uuid::new_v4();
    info!("abetal {}", &key);

    let (status_tx, status_rx) = mpsc::channel();
    status_pending.lock().unwrap().insert(key, status_tx);

    let sim_rx = if json.0.dryrun {
        let (sim_tx, sim_rx) = mpsc::channel();
        sim_pending.lock().unwrap().insert(key, sim_tx);
        Some(sim_rx)
    } else {
        None
    };

    kafka::produce_utbetaling(key, models::Utbetaling::Tp(&json.0)).await;

    let mut handlers = Vec::new();

    if let Some(sim_rx) = sim_rx {
        handlers.push(actix_web::rt::spawn(simulering_handler(sim_rx)));
    }

    handlers.push(actix_web::rt::spawn(status_handler(status_rx)));

    let (first_done, _idx, rest) = select_all(handlers).await;

    for h in rest {
        h.abort();
    }

    first_done.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
}

async fn simulering_handler(sim_rx: Receiver<Simulering>) -> HttpResponse {
    match sim_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(sim) => HttpResponse::Ok().json(sim),
        Err(mpsc::RecvTimeoutError::Timeout) => HttpResponse::RequestTimeout().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn status_handler(status_rx: Receiver<Reply>) -> HttpResponse {
    match status_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(reply) => match reply.status {
            Status::Ok => HttpResponse::Ok().json(reply.status),
            Status::Mottatt => HttpResponse::Accepted().json(reply.status),
            Status::HosOppdrag => HttpResponse::Accepted().json(reply.status),
            Status::Feilet => match reply.error {
                None => HttpResponse::InternalServerError().finish(),
                Some(error) => HttpResponseBuilder::new(
                    StatusCode::from_u16(error.status_code).expect("Valid status code"),
                )
                .json(error.msg),
            },
        },
        Err(mpsc::RecvTimeoutError::Timeout) => HttpResponse::RequestTimeout().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
