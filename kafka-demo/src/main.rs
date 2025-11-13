use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use clap::Parser;
use log::{error, info};
use logforth::{Layout, append::Stdout, filter::env_filter::EnvFilterBuilder};
use rdkafka::{
    ClientConfig,
    producer::{FutureProducer, FutureRecord},
};

#[derive(Parser)]
#[command(version, about = "Kafka Producer", long_about = None)]
struct Options {
    #[arg(long, default_value_t = String::from("localhost:9092"))]
    broker: String,

    #[arg(long)]
    topic: String,

    #[arg(long)]
    token: Option<String>,

    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[derive(Debug)]
struct CustomLayout;

impl Layout for CustomLayout {
    fn format(
        &self,
        record: &logforth::record::Record,
        _diags: &[Box<dyn logforth::Diagnostic>],
    ) -> Result<Vec<u8>, logforth::Error> {
        let base_file_name = record
            .file()
            .map(std::path::Path::new)
            .and_then(std::path::Path::file_name)
            .map(std::ffi::OsStr::to_string_lossy)
            .unwrap_or_default();
        Ok(format!(
            "{} [{}] {} {}:{} {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S.%3f"),
            std::thread::current().name().unwrap_or("unknown"),
            record.level().name(),
            base_file_name,
            record.line().unwrap_or(0),
            record.payload()
        )
        .into_bytes())
    }
}

#[tokio::main]
async fn main() {
    logforth::starter_log::builder()
        .dispatch(|d| {
            d.filter(EnvFilterBuilder::from_default_env().build())
                .append(Stdout::default().with_layout(CustomLayout))
        })
        .apply();
    let options = Options::parse();
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", &options.broker);
    config.set("message.timeout.ms", "30000");
    config.set("enable.idempotence", "true");
    // Disable batching
    config.set("batch.size", "1");
    config.set("linger.ms", "1");
    if let Some(token) = &options.token {
        config.set("sasl.mechanism", "PLAIN");
        config.set("security.protocol", "SASL_SSL");
        config.set("sasl.username", "user");
        config.set("sasl.password", format!("token:{token}"));
    }
    if options.debug {
        config.set("debug", "all");
    }
    let producer: FutureProducer = config.create().unwrap();
    // Trigger the producer state recovery
    let topic = options.topic;
    let value = "first".to_string();
    let record: FutureRecord<String, String> = FutureRecord::to(topic.as_str()).payload(&value);
    let start = std::time::Instant::now();
    let future = producer.send(record, Duration::from_secs(0));
    let result = future.await;
    let elapsed = start.elapsed().as_millis() as f64;
    match result {
        Ok(delivery) => info!("Sent 1st message to {:?} after {} ms", delivery, elapsed),
        Err(err) => error!("Failed to send after {} ms: {:?}", elapsed, err),
    };
    let running = Arc::new(AtomicBool::new(true));
    for i in 0..100 {
        if !running.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let producer = producer.clone();
        let topic = topic.clone();
        let running = running.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let value = format!("message {}", i);
            let record: FutureRecord<String, String> =
                FutureRecord::to(topic.as_str()).payload(&value);
            let future = producer.send(record, Duration::from_secs(0));
            let result = future.await;
            let elapsed = start.elapsed().as_millis() as f64;
            match result {
                Ok(delivery) => {
                    info!("Sent {} to {:?} after {} ms", i, delivery, elapsed);
                }
                Err(err) => {
                    error!("Failed to send {} after {} ms: {:?}", i, elapsed, err);
                    running.store(false, std::sync::atomic::Ordering::Relaxed);
                }
            };
        });
        if i % 5 == 4 {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
    tokio::time::sleep(Duration::from_millis(3000)).await;
}
