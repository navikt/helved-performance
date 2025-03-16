use std::{env, hash::Hasher, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, time::Duration};
use actix_web::rt::{spawn, task::JoinHandle, time::sleep};
use chrono::{DateTime, NaiveDate, Utc};
use log::{error, info};
use rdkafka::{consumer::{BaseConsumer, Consumer}, producer::{FutureProducer, FutureRecord}, ClientConfig, Message};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash32;
use uuid::Uuid;

const NUM_PARTITIONS: i32 = 3;

pub async fn abetal(uid: Uuid, utbet: &Utbetaling)
{
    let producer = producer("perf-abetal-aap");
    let key = uid.to_string();
    let value = serde_json::to_string(utbet).expect("failed to serialize");
    let record = FutureRecord::to("helved.aap-utbetalinger.v1")
        .key(&key)
        .payload(&value)
        .partition(partition(uid));

    match producer.send(record, Duration::from_secs(5)).await {
        Ok(delivery) => info!("Record sent: {:?}", delivery),
        Err((err, msg)) => error!("Failed to send record: {:?} msg: {:?}", err, msg),
    };
}

pub fn init_status_consumer() -> (Arc<Channel<StatusReply>>, JoinHandle<()>)
{
    let channel = Arc::new(Channel::default());
    let handle = spawn(status_consumer(channel.clone()));
    (channel.clone(), handle)
}

async fn status_consumer(channel: Arc<Channel<StatusReply>>) {
    let consumer = consumer("perf-abetal-status");
    consumer.subscribe(&["helved.status.v1"]).expect("subscribe to status-topic");
    let mut current_uid: Option<Uuid> = None;
    loop {
        if !*channel.active.lock().unwrap() {
            break;
        }
        {
            let rx = &channel.uid.lock().unwrap().1;
            if let Ok(uid) = rx.recv_timeout(Duration::from_millis(10)) {
                current_uid = Some(uid);
            }
        }
        if let Some(uid) = current_uid {
            if let Some(result) = consumer.poll(Duration::from_millis(10)) {
                let record = result.expect("kafka message");
                if uid.to_string() != std::str::from_utf8(record.key().expect("key")).expect("str") { 
                    continue 
                };
                let payload = record.payload().expect("payload");
                let json = std::str::from_utf8(payload).unwrap();
                info!("Record received: {}", json);
                let state: StatusReply = serde_json::from_str(json).unwrap(); 
                {
                    let tx = &channel.result.lock().unwrap().0;
                    tx.send(state.clone()).unwrap();
                }
                match state.status {
                    Status::Ok => current_uid = None,
                    Status::Feilet => current_uid = None,
                    _ => {},
                } 
            };
        };

        sleep(Duration::from_millis(1)).await;
    }

    consumer.unsubscribe();
}

pub fn init_aap_simulering_consumer() -> (Arc<Channel<Simulering>>, JoinHandle<()>)
{
    let channel = Arc::new(Channel::default());
    let handle = spawn(aap_simulering_consumer(channel.clone()));
    (channel.clone(), handle)
}

async fn aap_simulering_consumer(channel: Arc<Channel<Simulering>>) {
    let consumer = consumer("perf-aap-simulering");
    consumer.subscribe(&["helved.aap-simulering.v1"]).expect("subscribe to aap-simulering-topic");
    let mut current_uid: Option<Uuid> = None;
    loop {
        if !*channel.active.lock().unwrap() {
            break;
        }
        {
            let rx = &channel.uid.lock().unwrap().1;
            if let Ok(uid) = rx.recv_timeout(Duration::from_millis(10)) {
                current_uid = Some(uid);
            }
        }
        if let Some(uid) = current_uid {
            if let Some(result) = consumer.poll(Duration::from_millis(10)) {
                let record = result.expect("kafka message");
                if uid.to_string() != std::str::from_utf8(record.key().expect("key")).expect("str") { 
                    continue 
                };
                let payload = record.payload().expect("payload");
                let json = std::str::from_utf8(payload).unwrap();
                info!("Record received: {}", json);
                let simulering: Simulering = serde_json::from_str(json).unwrap(); 
                let tx = &channel.result.lock().unwrap().0;
                tx.send(simulering.clone()).unwrap();
                current_uid = None
            };
        };

        sleep(Duration::from_millis(1)).await;
    }

    consumer.unsubscribe();
}

fn producer(client_id: &str) -> FutureProducer
{
    ClientConfig::new()
        .set("bootstrap.servers", env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS"))
        .set("client.id", client_id.to_owned())
        .set("security.protocol", "ssl")
        .set("compression.codec", "snappy")
        .set("ssl.key.location", env::var("KAFKA_PRIVATE_KEY_PATH").expect("KAFKA_PRIVATE_KEY_PATH"))
        .set("ssl.certificate.location", env::var("KAFKA_CERTIFICATE_PATH").expect("KAFKA_CERTIFICATE_PATH"))
        .set("ssl.ca.location", env::var("KAFKA_CA_PATH").expect("KAFKA_CA_PATH"))
        .create()
        .unwrap_or_else(|_| { 
                error!("Failed to create kafka producer {client_id}");
                panic!("Failed to create kafka producer {client_id}")
            }
        )
}

fn consumer(client_id: &str) -> BaseConsumer  
{
    ClientConfig::new()
        .set("bootstrap.servers", env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS"))
        .set("client.id", client_id.to_owned())
        .set("group.id", format!("{}-consumer", &client_id))
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "false")
        .set("session.timeout.ms", "6000")
        .set("security.protocol", "ssl")
        .set("ssl.key.location", env::var("KAFKA_PRIVATE_KEY_PATH").expect("KAFKA_PRIVATE_KEY_PATH"))
        .set("ssl.certificate.location", env::var("KAFKA_CERTIFICATE_PATH").expect("KAFKA_CERTIFICATE_PATH"))
        .set("ssl.ca.location", env::var("KAFKA_CA_PATH").expect("KAFKA_CA_PATH"))
        .create()
        .unwrap_or_else(|_| panic!("Failed to create kafka consumer {client_id}"))
}

fn partition(key: Uuid) -> i32
{
    let mut hasher = XxHash32::with_seed(0); // seed 0 like kakfa's murmur2
    hasher.write(key.to_string().as_bytes());
    let hash = hasher.finish() as i32;
    (hash & 0x7fffffff) % NUM_PARTITIONS // ensure positive result
}

pub struct Channel<T> {
    active: Mutex<bool>,
    pub result: Mutex<(Sender<T>, Receiver<T>)>,
    pub uid: Mutex<(Sender<Uuid>, Receiver<Uuid>)>,
}

impl <T> Channel<T> {
    pub fn disable(&self) {
        *self.active.lock().unwrap() = false
    }
}

impl <T> Default for Channel<T> {
    fn default() -> Self {
        Channel {
            active: Mutex::new(true),
            result: Mutex::new(mpsc::channel()),
            uid: Mutex::new(mpsc::channel()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetaling 
{
    pub simulate: bool,
    action: Action,
    sak_id: String,
    behandling_id: String,
    personident: String,
    vedtakstidspunkt: DateTime<Utc>,
    stønad: String,
    beslutter_id: String,
    saksbehandler_id: String,
    periodetype: Periodetype,
    perioder: Vec<Utbetalingsperiode>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetalingsperiode {
    fom: NaiveDate,
    tom: NaiveDate,
    beløp: u32,
    betalende_enhet: Option<String>,
    fastsatt_dagsats: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Action {
    Create,
    Update,
    Delete,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Periodetype {
    Dag,
    Ukedag,
    Mnd,
    EnGang,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusReply {
    pub status: Status,
    error: Option<ApiError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    status_code: i32,
    msg: String,
    doc: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Ok,
    Feilet,
    Mottatt,
    HosOppdrag,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Simulering {
    perioder: Vec<SimuleringPeriode>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimuleringPeriode {
    fom: NaiveDate,
    tom: NaiveDate,
    utbetalinger: Vec<SimuleringUtbetaling>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimuleringUtbetaling {
    fagsystem: String,
    sak_id: String,
    utbetales_til: String,
    stønadstype: String,
    tidligere_utbetalt: i32,
    nytt_beløp: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_consistency() {
        let key = Uuid::new_v4();
        let p1 = partition(key);
        let p2 = partition(key);
        assert_eq!(p1, p2, "Partitioning should be consistent for the same key");
    }
}
