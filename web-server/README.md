# web-server

It's based on https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html but with some modifications:

- Each `Worker` has its own sender and receiver pair, while the original project shares the receiver among all workers.
- Hence, the thread pool chooses threads in a round-robin way rather than relying on the behavior of `mpsc::channel()`.
- Support pressing `Ctrl+C` to exit.

## Usage

Navigate to the project directory and run:

```bash
cargo build
./target/debug/web-server
```

Then it will try listening on port 7878. You can also specify an alternative URL like `./target/debug/web-server localhost:9999`.

Open a new terminal and run the client Python script to see the behavior:

```bash
python3 web-server/client.py
```

Finally, press Ctrl+C to exit the server.

Example outputs when the server exited after running the client script twice:

```
...
2025-06-16 22:32:31.066 - worker-0 - Worker 0 received a job
2025-06-16 22:32:31.171 - worker-1 - Worker 1 received a job
2025-06-16 22:32:31.277 - worker-2 - Worker 2 received a job
2025-06-16 22:32:32.008 - worker-3 - Worker 3 received a job
2025-06-16 22:32:32.113 - worker-0 - Worker 0 received a job
2025-06-16 22:32:32.219 - worker-1 - Worker 1 received a job
^C2025-06-16 22:32:34.253 - ctrl-c - The server is shutting down and will exit after accepting another connection
...
```
