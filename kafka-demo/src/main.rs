use std::time::Duration;

use clap::Parser;
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

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", &options.broker);
    config.set("message.timeout.ms", "10000");
    config.set("enable.idempotence", "true");
    if let Some(token) = &options.token {
        config.set("sasl.mechanism", "PLAIN");
        config.set("security.protocol", "SASL_SSL");
        config.set("sasl.username", "user");
        config.set("sasl.password", format!("token:{token}"));
    }
    if options.debug {
        env_logger::init();
        config.set("debug", "all");
    }
    let producer: FutureProducer = config.create().unwrap();
    // Send 10 messages synchronously
    for i in 0..10 {
        let start = std::time::Instant::now();
        let value = format!("message {}", i);
        let record: FutureRecord<String, String> =
            FutureRecord::to(options.topic.as_str()).payload(&value);
        let result = producer.send(record, Duration::from_secs(0)).await;
        let elapsed = start.elapsed().as_millis() as f64;
        match result {
            Ok(delivery) => println!("Sent to {:?} after {} ms", delivery, elapsed),
            Err(err) => println!("Failed to send after {} ms: {:?}", elapsed, err),
        };
    }
}
