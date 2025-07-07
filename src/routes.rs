use std::time::{Duration, Instant};
use actix_web::rt::time::sleep;
use actix_web::web::{self, Data};
use actix_web::{get, post, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::kafka;
use crate::models;

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/abetal")]
pub async fn abetal(
    status: Data<kafka::Channel<models::status::Reply>>,
    simulering: Data<kafka::Channel<models::dryrun::Simulering>>,
    json: web::Json<models::dp::Utbetaling>,
) -> HttpResponse {
    let key = Uuid::new_v4();
    info!("abetal {}", &key);

    {
        let tx = &status.uid.lock().unwrap().0;
        tx.send(key).unwrap();
    }

    if json.0.dryrun {
        let tx = &simulering.uid.lock().unwrap().0;
        tx.send(key).unwrap();
    }

    kafka::abetal(key, &json.0).await; 

    let start = Instant::now();
    let mut last_status: Option<models::status::Reply> = None;
    let mut simulering_result: Option<models::dryrun::Simulering> = None;
    loop {
        if start.elapsed() >= Duration::from_secs(50) { // simulering kan være treg
            break;
        }
        {
            if json.0.dryrun {
                let rx = &simulering.result.lock().unwrap().1;
                if let Ok(rec) = rx.recv_timeout(Duration::from_millis(10)) {
                    simulering_result = Some(rec.clone());
                    break;
                }
            }

            let rx = &status.result.lock().unwrap().1;
            if let Ok(rec) = rx.recv_timeout(Duration::from_millis(10)) {
                last_status = Some(rec.clone());
                match rec.status {
                    models::status::Status::Mottatt    => {},
                    models::status::Status::HosOppdrag => {},
                    models::status::Status::Feilet     => break,
                    models::status::Status::Ok         => break,
                }
            }
        }
        sleep(Duration::from_millis(1)).await;
    }

    if let Some(sim) = simulering_result {
        return HttpResponse::Ok().json(sim);
    };

    match last_status {
        Some(status) => HttpResponse::Ok().json(status),
        None => HttpResponse::Accepted().finish(),
    }
}

