use crate::models::Utbetaling;
use crate::{
    models::{self, status},
    routes::{StatusPubSub, SimPubSub},
};
use actix_web::rt::time::sleep;
use tokio::sync::mpsc;
use log::{error, info};
use rdkafka::{
    ClientConfig, Message,
    consumer::{BaseConsumer, Consumer},
    producer::{FutureProducer, FutureRecord},
};
use std::{env, hash::Hasher, time::Duration};
use twox_hash::XxHash32;
use uuid::Uuid;

const NUM_PARTITIONS: i32 = 3;

pub async fn produce_utbetaling(uid: Uuid, utbet: Utbetaling) {
    {
        let (topic, producer, value) = match utbet {
            Utbetaling::Aap(aap) => {
                let topic = "helved.utbetalinger-aap.v1";
                let producer = producer("produce-aap-utbetaling");
                let value = serde_json::to_string(&aap).expect("failed to serialize");
                (topic, producer, value)
            }
            Utbetaling::Dp(dp) => {
                let topic = "helved.utbetalinger-dp.v1";
                let producer = producer("produce-dp-utbetaling");
                let value = serde_json::to_string(&dp).expect("failed to serialize");
                (topic, producer, value)
            }
            Utbetaling::Ts(ts) => {
                let topic = "helved.utbetalinger-ts.v1";
                let producer = producer("produce-ts-utbetaling");
                let value = serde_json::to_string(&ts).expect("failed to serialize");
                (topic, producer, value)
            }
            Utbetaling::Tp(tp) => {
                let topic = "helved.utbetalinger-tp.v1";
                let producer = producer("produce-tp-utbetaling");
                let value = serde_json::to_string(&tp).expect("failed to serialize");
                (topic, producer, value)
            },
            Utbetaling::Historisk(historisk) => {
                let topic = "helved.utbetalinger-historisk.v1";
                let producer = producer("produce-historisk-utbetaling");
                let value = serde_json::to_string(&historisk).expect("failed to serialize");
                (topic, producer, value)
            }
        };

        let key = uid.to_string();
        let record = FutureRecord::to(topic)
            .key(&key)
            .payload(&value)
            .partition(partition(uid));

        match producer.send(record, Duration::from_secs(5)).await {
            Ok(delivery) => info!("Record sent: {:?}", delivery),
            Err((err, msg)) => error!("Failed to send record: {:?} msg: {:?}", err, msg),
        };
    }
}

pub async fn status_consumer(channel: StatusPubSub) {
    let consumer = consumer("consume_status");
    consumer
        .subscribe(&["helved.status.v1"])
        .expect("subscribe to status-topic");

    loop {
        if let Some(result) = consumer.poll(Duration::from_millis(50)) {
            let record = match result {
                Ok(record) => record,
                Err(e) => {
                    error!("failed to read record on helved.status.v1 {:?}", e);
                    continue;
                }
            };

            let uid = match record
                .key()
                .and_then(|it| std::str::from_utf8(it).ok())
                .and_then(|it| Uuid::parse_str(it).ok())
            {
                Some(uid) => uid,
                None => continue,
            };

            let payload = match record.payload().and_then(|it| std::str::from_utf8(it).ok()) {
                Some(payload) => payload,
                None => continue,
            };

            let reply: status::Reply = match serde_json::from_str(payload) {
                Ok(reply) => reply,
                Err(e) => {
                    error!("failed to deserialize status::Reply {:?}", e);
                    continue;
                }
            };

            if let Some((tx, _)) = channel.lock().await.get(&uid) {
                let _ = tx.send(reply).await;
            }
        }

        {
            let mut map = channel.lock().await;
            let mut uids_to_remove = Vec::new();

            for (uid, (_, rx)) in map.iter_mut() {
                match rx.try_recv() {
                    Ok(_) => {
                        info!("Cleanup signal received for UID: {}", uid);
                        uids_to_remove.push(*uid);
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        info!("Router sender disconnected fro UID: {}", uid);
                        uids_to_remove.push(*uid);
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // channel is empty, continue waiting
                    }
                }
            }

            for uid in uids_to_remove {
                map.remove(&uid);
                info!("Cleane up channel for UID: {}", uid);
            }
        }
    }

    // consumer.unsubscribe();
}

pub async fn dryrun_consumer(simulering_pending: SimPubSub) {
    let consumer = consumer("consume-dryruns");
    consumer
        .subscribe(&["helved.dryrun-aap.v1", "helved.dryrun-dp.v1"])
        .expect("subscribe to topic dryrun-aap or dryrun-dp");

    loop {
        if let Some(result) = consumer.poll(Duration::from_millis(50)) {
            let record = match result {
                Ok(record) => record,
                Err(e) => {
                    error!(
                        "failed to read record on helved.dryrun-aap.v1 or helved.dryrun-dp.v1 {:?}",
                        e
                    );
                    continue;
                }
            };

            let uid = match record
                .key()
                .and_then(|it| std::str::from_utf8(it).ok())
                .and_then(|it| Uuid::parse_str(it).ok())
            {
                Some(uid) => uid,
                None => continue,
            };

            let payload = match record.payload().and_then(|it| std::str::from_utf8(it).ok()) {
                Some(payload) => payload,
                None => continue,
            };

            let simulering: models::dryrun::Simulering = match serde_json::from_str(payload) {
                Ok(simulering) => simulering,
                Err(e) => {
                    error!("failed to deserialize models::dryrun::Simulering {:?}", e);
                    continue;
                }
            };

            if let Some(simulering_tx) = simulering_pending.lock().await.remove(&uid) {
                let _ = simulering_tx.send(simulering).await;
            }
        }

        sleep(Duration::from_millis(1)).await;
    }

    // consumer.unsubscribe();
}

fn producer(client_id: &str) -> FutureProducer {
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
        })
}

fn consumer(client_id: &str) -> BaseConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS"))
        .set("client.id", client_id.to_owned())
        .set("group.id", format!("{}-consumer", &client_id))
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "false")
        .set("socket.keepalive.enable", "true")
        .set("session.timeout.ms", "90000")
        .set("heartbeat.interval.ms", "10000")
        .set("security.protocol", "ssl")
        .set("ssl.key.location", env::var("KAFKA_PRIVATE_KEY_PATH").expect("KAFKA_PRIVATE_KEY_PATH"))
        .set("ssl.certificate.location", env::var("KAFKA_CERTIFICATE_PATH").expect("KAFKA_CERTIFICATE_PATH"))
        .set("ssl.ca.location", env::var("KAFKA_CA_PATH").expect("KAFKA_CA_PATH"))
        .create()
        .unwrap_or_else(|_| panic!("Failed to create kafka consumer {client_id}"))
}

fn partition(key: Uuid) -> i32 {
    let mut hasher = XxHash32::with_seed(0); // seed 0 like kakfa's murmur2
    hasher.write(key.to_string().as_bytes());
    let hash = hasher.finish() as i32;
    (hash & 0x7fffffff) % NUM_PARTITIONS // ensure positive result
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
