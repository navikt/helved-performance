use actix_web::http::StatusCode;
use actix_web::web::{self, Data};
use actix_web::{HttpResponse, get, post};
use futures::future::select_all;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{self, Duration};
use std::{
    collections::HashMap,
    sync::Arc,
};
use uuid::Uuid;

use crate::kafka;
use crate::models;
use crate::models::dryrun::Simulering;
use crate::models::status::{Reply, Status, Error};

pub type StatusPubSub = Arc<Mutex<HashMap<Uuid, (mpsc::Sender<models::status::Reply>, mpsc::Receiver<Uuid>)>>>;
pub type SimPubSub = Arc<Mutex<HashMap<Uuid, mpsc::Sender<models::dryrun::Simulering>>>>;

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/abetal/aap")]
pub async fn abetal_aap(
    status_pubsub: Data<StatusPubSub>,
    sim_pubsub: Data<SimPubSub>,
    json: web::Json<models::aap::Utbetaling>,
) -> HttpResponse {
    let dryrun = json.0.dryrun.unwrap_or(false);
    let tx = Uuid::new_v4();
    handle_utbetaling(status_pubsub, sim_pubsub, json.0, dryrun, tx).await
}

#[post("/abetal/dp")]
pub async fn abetal_dp(
    status_pubsub: Data<StatusPubSub>,
    sim_pubsub: Data<SimPubSub>,
    json: web::Json<models::dp::Utbetaling>,
) -> HttpResponse {
    let dryrun = json.0.dryrun.unwrap_or(false);
    let tx = Uuid::new_v4();
    handle_utbetaling(status_pubsub, sim_pubsub, json.0, dryrun, tx).await
}

#[post("/abetal/dp/{transaction_id}")]
pub async fn abetal_dp_tx(
    status_pubsub: Data<StatusPubSub>,
    sim_pubsub: Data<SimPubSub>,
    json: web::Json<models::dp::Utbetaling>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let dryrun = json.0.dryrun.unwrap_or(false);
    let tx: Uuid = path.into_inner();
    handle_utbetaling(status_pubsub, sim_pubsub, json.0, dryrun, tx).await
}

#[post("/abetal/ts")]
pub async fn abetal_ts(
    status_pubsub: Data<StatusPubSub>,
    sim_pubsub: Data<SimPubSub>,
    json: web::Json<models::ts::Utbetaling>,
) -> HttpResponse {
    let dryrun = json.0.dryrun.unwrap_or(false);
    let tx = Uuid::new_v4();
    handle_utbetaling(status_pubsub, sim_pubsub, json.0, dryrun, tx).await
}

#[post("/abetal/tp")]
pub async fn abetal_tp(
    status_pubsub: Data<StatusPubSub>,
    sim_pubsub: Data<SimPubSub>,
    json: web::Json<models::tp::Utbetaling>,
) -> HttpResponse {
    let dryrun = json.0.dryrun.unwrap_or(false);
    let tx = Uuid::new_v4();
    handle_utbetaling(status_pubsub, sim_pubsub, json.0, dryrun, tx).await
}

async fn handle_utbetaling<T>(
    status_pubsub: web::Data<StatusPubSub>,
    sim_pubsub: web::Data<SimPubSub>,
    utbetaling: T,
    dryrun: bool,
    transaction_id: Uuid,
) -> HttpResponse 
where 
    T: Into<models::Utbetaling> + Clone,
{
    let (status_tx, status_rx) = mpsc::channel(100);
    let (actuator_tx, actuator_rx) = mpsc::channel(100);
    status_pubsub.lock().await.insert(transaction_id, (status_tx, actuator_rx));

    let mut sim_rx_opt = None;
    if dryrun {
        let (sim_tx, sim_rx) = mpsc::channel(100);
        sim_pubsub.lock().await.insert(transaction_id, sim_tx);
        sim_rx_opt = Some(sim_rx)
    }

    kafka::produce_utbetaling(transaction_id, utbetaling.into()).await;

    let mut handlers = Vec::new();

    if let Some(sim_rx) = sim_rx_opt {
        handlers.push(actix_web::rt::spawn(simulering_handler(sim_rx)));
    }

    handlers.push(actix_web::rt::spawn(status_handler(transaction_id, status_rx, actuator_tx)));

    let (first_done, _idx, rest) = select_all(handlers).await;

    for h in rest {
        h.abort();
    }

    first_done.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
}

async fn simulering_handler(mut sim_rx: mpsc::Receiver<Simulering>) -> HttpResponse {
    let timeout_duration = Duration::from_secs(30);
    let result = time::timeout(timeout_duration, sim_rx.recv()).await;
    match result {
        Ok(Some(sim)) => HttpResponse::Ok().json(sim),
        _ => {
            let timeout_error = Reply {
                status: Status::Feilet,
                error: Some(Error {
                    status_code: 408,
                    msg: "Fikk ingen response på simulering innen 30 sec".into(),
                    doc: "https://helved-docs.ansatt.dev.nav.no/v3/doc/".into(),
                }),
            };
            HttpResponse::RequestTimeout().json(timeout_error)
        }
    }
}

async fn status_handler(
    uid: Uuid,
    status_rx: mpsc::Receiver<Reply>,
    actuator_tx: mpsc::Sender<Uuid>,
) -> HttpResponse {
    let timeout_duration = Duration::from_secs(30);
    let monitor_future = monitor_replies(status_rx);
    let result = time::timeout(timeout_duration, monitor_future).await;

    let _ = actuator_tx.send(uid).await; // unsubscribe uid

    match result {
        Ok(Some(reply)) => {
            match reply.status {
                Status::Ok => HttpResponse::Ok().json(reply),
                Status::Feilet => {
                    match reply.error {
                        None => HttpResponse::InternalServerError().json(reply),
                        Some(ref error) => {
                            let status_code = StatusCode::from_u16(error.status_code).unwrap_or(StatusCode::BAD_REQUEST);
                            HttpResponse::build(status_code).json(reply)
                        }
                    }
                }
                _ => HttpResponse::InternalServerError().finish()
            }
        }
        Err(_) => {
            let timeout_error = Reply {
                status: Status::Feilet,
                error: Some(Error {
                    status_code: 408,
                    msg: "Fikk ingen endelig status innen 30 sec".into(),
                    doc: "https://helved-docs.ansatt.dev.nav.no/v3/doc/".into(),
                }),
            };
            HttpResponse::RequestTimeout().json(timeout_error)
        }
        Ok(None) => {
            let closed_error = Reply {
                status: Status::Feilet,
                error: Some(Error {
                    status_code: 500,
                    msg: "Channel closed unexpectedly".into(),
                    doc: "https://helved-docs.ansatt.dev.nav.no/v3/doc/".into(),
                }),
            };
            HttpResponse::InternalServerError().json(closed_error)
        }
    }
}

async fn monitor_replies(mut status_rx: mpsc::Receiver<Reply>) -> Option<Reply> {
    let mut last_reply: Option<Reply> = None;
    loop {
        match status_rx.recv().await {
            Some(res) => {
                match res.status {
                    Status::Ok | Status::Feilet => {
                        return Some(res);
                    }
                    _ => {
                        last_reply = Some(res);
                        continue;
                    }
                }
            }
            None => return last_reply
        }
    }
}

