# tokio-demo

https://tokio.rs/tokio/tutorial

## Usage

Navigate to the project directory.

Run the server:

```bash
cargo run --package tokio-demo server
```

By default, it listens at `127.0.0.1:6379`, you can change it by the `--url` option.

Run the client:

```bash
cargo run --package tokio-demo --example hello-redis
```
