use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, value: Bytes },
    Quit,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key } => match client.get(&key).await.unwrap() {
                    Some(bytes) => println!(
                        "Got value for key {}: {:?}",
                        key,
                        String::from_utf8(bytes.to_vec()).unwrap()
                    ),
                    None => println!("No value found for key {}", key),
                },
                Command::Set { key, value } => {
                    println!("Set key: {}, value: {:?}", &key, &value);
                    client.set(&key, value).await.unwrap();
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
        tx1.send(Command::Get {
            key: String::from("foo"),
        })
        .await
        .unwrap();
    });
    let tx2 = tx.clone();
    let handle2 = tokio::spawn(async move {
        tx2.send(Command::Set {
            key: String::from("foo"),
            value: "bar".into(),
        })
        .await
        .unwrap();
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    tx.send(Command::Quit).await.unwrap();
    manager.await.unwrap();
}
