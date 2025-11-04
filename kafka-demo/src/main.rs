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
    config.set("message.timeout.ms", "5000");
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
    let value = String::from("msg");
    let record: FutureRecord<String, String> =
        FutureRecord::to(options.topic.as_str()).payload(&value);
    match producer.send(record, Duration::from_secs(0)).await {
        Ok(result) => println!("Sent to {:?}", result),
        Err(err) => println!("Failed to send: {:?}", err),
    };
}
