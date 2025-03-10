use std::{collections::HashMap, env, hash::Hasher, time::{Duration, Instant}};
use actix_web::rt::spawn;
use chrono::{DateTime, NaiveDate, Utc};
use log::{error, info};
use rdkafka::{consumer::{BaseConsumer, Consumer}, producer::{FutureProducer, FutureRecord}, ClientConfig, Message, Offset, TopicPartitionList};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash32;
use uuid::Uuid;

const NUM_PARTITIONS: i32 = 3;

pub async fn abetal(uid: Uuid, utbet: Utbetaling) -> Option<StatusReply>
{
    let producer = producer("perf-abetal-aap");
    let key = uid.to_string();
    let value = serde_json::to_string(&utbet).expect("failed to serialize");
    let record = FutureRecord::to("helved.aap-utbetalinger.v1")
        .key(&key)
        .payload(&value)
        .partition(partition(uid));

    match producer.send(record, Duration::from_secs(5)).await {
        Ok(delivery) => info!("Record sent: {:?}", delivery),
        Err((err, msg)) => error!("Failed to send record: {:?} msg: {:?}", err, msg),
    };

    let handle = spawn(wait_for_status(uid));
    handle.await.unwrap()
}

async fn wait_for_status(uid: Uuid) -> Option<StatusReply> {
    let start = Instant::now();
    let consumer = consumer("perf-abetal-status");
    consumer.subscribe(&["helved.status.v1"]).expect("subscribe to status-topic");
    let topic_map = HashMap::from([(("helved.status.v1".into(), partition(uid)), Offset::End)]);
    let tpl = TopicPartitionList::from_topic_map(&topic_map).expect("topic partition list");
    consumer.assign(&tpl).expect("assign offset to status-topic");
    consumer.seek("helved.status.v1", partition(uid), Offset::End, Duration::from_secs(1)).unwrap();
    let mut last_state: Option<StatusReply> = None;
    loop {
        match consumer.poll(Duration::from_secs(1)) {
            Some(result) => {
                let record = result.expect("kafka message");
                if uid.to_string() != std::str::from_utf8(record.key().expect("key")).expect("str") { continue };
                let payload = record.payload().expect("payload");
                let json = std::str::from_utf8(payload).unwrap();
                info!("Record received: {}", json);
                last_state = Some(serde_json::from_str(json).unwrap()); 
                match last_state.clone().unwrap().status  {
                    Status::Ok | Status::Feilet => {
                        consumer.unsubscribe();
                        return last_state;
                    },
                    _ => {}
                }
            },
            None => {
                if start.elapsed() >= Duration::from_secs(30) {
                    consumer.unsubscribe();
                    return last_state;
                }
            },
        };
    }
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
        .unwrap_or_else(|_| panic!("Failed to create kafka producer {client_id}"))
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
        .set("compression.codec", "snappy")
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetaling 
{
    simulate: bool,
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
    status: Status,
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
