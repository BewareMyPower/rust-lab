use std::collections::HashMap;

use mini_redis::{Command, Connection, Frame, Result};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "127.0.0.1:6379";
    let listener = TcpListener::bind(url).await.unwrap();
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut db = HashMap::<String, Vec<u8>>::new();
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            },
            Command::Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
