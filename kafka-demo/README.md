# kafka-demo

## Run the example

The example just sends a message via an idempotent producer.

```bash
cargo run -- --broker <broker-url> --topic <topic-name>
```

To enable debug logs from rdkafka, set the `RUST_LOG` environment variable and add the `--debug` option, for example:

```bash
RUST_LOG="librdkafka=trace,rdkafka::client=debug" cargo run -- --broker localhost:9092 --topic my-topic --debug
```

If your target server has enabled token based authentication, add the `--token` option to specify the token value.
