use std::{env, hash::Hasher, time::Duration};
use chrono::{DateTime, NaiveDate, Utc};
use log::{error, info};
use rdkafka::{producer::{FutureProducer, FutureRecord}, ClientConfig};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash32;
use uuid::Uuid;

const NUM_PARTITIONS: i32 = 3;

pub async fn abetal(uid: Uuid, utbet: Utbetaling)
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
        .expect("Failed to create kafka producer perf-abetal")
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
