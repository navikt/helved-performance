use std::{env, hash::{DefaultHasher, Hash, Hasher}, time::Duration};

use chrono::{DateTime, NaiveDate, Utc};
use log::{error, info};
use rdkafka::{producer::{FutureProducer, FutureRecord}, ClientConfig};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    let mut hasher = DefaultHasher::new();
    key.to_string().hash(&mut hasher);
    (hasher.finish() % 3) as i32
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetaling 
{
    simulate: bool,
    action: Action,
    sak_id: SakId,
    behandling_id: BehandlingId,
    personident: Personident,
    vedtakstidspunkt: DateTime<Utc>,
    stønad: String,
    beslutter_id: Navident,
    saksbehandler_id: Navident,
    periodetype: Periodetype,
    perioder: Vec<Utbetalingsperiode>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetalingsperiode {
    fom: NaiveDate,
    tom: NaiveDate,
    beløp: u32,
    betalende_enhet: Option<NavEnhet>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SakId { id: String }

#[derive(Serialize, Deserialize, Debug)]
pub struct BehandlingId { id: String }

#[derive(Serialize, Deserialize, Debug)]
pub struct Personident { ident: String }

#[derive(Serialize, Deserialize, Debug)]
pub struct Navident { ident: String }

#[derive(Serialize, Deserialize, Debug)]
pub struct NavEnhet { enhet: String }

