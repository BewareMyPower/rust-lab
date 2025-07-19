use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{Command, Frame, Result};
use tokio::net::TcpListener;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

pub(crate) async fn run_server(url: &str) -> Result<()> {
    let listener = TcpListener::bind(url).await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::new()));

    println!("Mini Redis Server running on {}", url);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: tokio::net::TcpStream, db: Db) {
    let mut connection = mini_redis::Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
