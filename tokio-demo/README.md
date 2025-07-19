# tokio-demo

https://tokio.rs/tokio/tutorial

## Usage

Navigate to the project directory.

Run the server:

```bash
cargo run --package tokio-demo server
```

By default, it listens at `127.0.0.1:6379`, you can change it by the `--url` option.

Run a single client command:

```bash
cargo run --package tokio-demo client get key
cargo run --package tokio-demo client set key value
cargo run --package tokio-demo client get key
```

The outputs will be:

```
No value found for key "key"
Set "key" => "value" in Redis
Got value "value" for key "key"
```

Run the client example with composite commands, which:
1. Set the `hello => world` key-value pair.
2. Get the value of key `hello`.
2. Get the value of key `key`.

```bash
cargo run --package tokio-demo --example hello-redis
```
