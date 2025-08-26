use std::{time::Duration, collections::HashMap, sync::{Arc, Mutex, mpsc}};
use actix_web::web::{self, Data};
use actix_web::{get, post, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::kafka;
use crate::models;

pub type PendingMap<T> = Arc<Mutex<HashMap<Uuid, mpsc::Sender<T>>>>; 

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
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

    kafka::produce_dp_utbetaling(key, &json.0).await; 

    if let Some(sim_rx) = sim_rx {
        match sim_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(sim) => return HttpResponse::Ok().json(sim),
            Err(mpsc::RecvTimeoutError::Timeout) => return HttpResponse::RequestTimeout().finish(),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    match status_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(mpsc::RecvTimeoutError::Timeout) => HttpResponse::RequestTimeout().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

