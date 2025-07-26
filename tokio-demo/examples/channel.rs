use bytes::Bytes;
use mini_redis::{Result, client};
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        responder: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        responder: Responder<()>,
    },
    Quit,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, responder } => {
                    let result = client.get(&key).await;
                    let _ = responder.send(result);
                }
                Command::Set {
                    key,
                    value,
                    responder,
                } => {
                    let result = client.set(&key, value).await;
                    let _ = responder.send(result);
                }
                Command::Quit => {
                    println!("Shutting down manager");
                    break;
                }
            }
        }
    });

    let tx1 = tx.clone();
    let handle1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx1.send(Command::Get {
            key: String::from("foo"),
            responder: resp_tx,
        })
        .await
        .unwrap();
        match resp_rx.await.unwrap().unwrap() {
            Some(bytes) => println!("Got value: {}", String::from_utf8(bytes.to_vec()).unwrap()),
            None => println!("Key not found"),
        }
    });
    let tx2 = tx.clone();
    let handle2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set {
            key: String::from("foo"),
            value: "bar".into(),
            responder: resp_tx,
        })
        .await
        .unwrap();
        let result = resp_rx.await.unwrap().unwrap();
        println!("Set foo => bar: {:?}", result);
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    tx.send(Command::Quit).await.unwrap();
    manager.await.unwrap();
}
