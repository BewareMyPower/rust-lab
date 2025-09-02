use futures::channel::mpsc;
use futures::future::Either;
use futures::{SinkExt, StreamExt, future};
use std::collections::VecDeque;
use std::{thread, time::Duration};

const MAX_LENGTH: usize = 5;
const BATCH_TIMEOUT: Duration = Duration::from_secs(1);

macro_rules! log {
    ($($arg:tt)*) => {
        let now = chrono::Local::now();
        print!("{} - {} - ", now.format("%Y-%m-%d %H:%M:%S%.3f"), thread::current().name().unwrap_or("unknown"));
        println!($($arg)*);
    };
}

struct Producer;

impl Producer {
    fn send(&self, messages: Vec<String>) {
        if !messages.is_empty() {
            log!("Sending messages: {:?}", messages);
        }
    }
}

impl Clone for Producer {
    fn clone(&self) -> Self {
        Producer {}
    }
}

struct Batch {
    sender: mpsc::Sender<String>,
}

impl Batch {
    fn new(producer: Producer) -> Batch {
        let (sender, mut receiver) = mpsc::channel::<String>(MAX_LENGTH);
        let producer = producer.clone();
        tokio::spawn(async move {
            let mut storage = BatchStorage::new();
            let mut recv_future = receiver.next();
            let mut timer_future = Box::pin(tokio::time::sleep(Duration::from_secs(1)));
            loop {
                match future::select(recv_future, timer_future).await {
                    Either::Left((Some(msg), _)) => {
                        storage.push_back(msg);
                        if storage.len() >= MAX_LENGTH {
                            let messages = storage.get_messages();
                            producer.send(messages);
                        }
                        recv_future = receiver.next();
                        timer_future = Box::pin(tokio::time::sleep(BATCH_TIMEOUT));
                    }
                    Either::Left((None, _)) => {
                        log!(
                            "Channel closed (unsent messages: {:?})",
                            storage.get_messages()
                        );
                        return;
                    }
                    Either::Right((_, recv)) => {
                        log!("Timeout reached");
                        producer.send(storage.get_messages());
                        recv_future = recv; // This is already the correct type.
                        timer_future = Box::pin(tokio::time::sleep(BATCH_TIMEOUT));
                    }
                }
            }
        });
        Batch { sender }
    }

    async fn push(&mut self, msg: String) {
        let mut sender = self.sender.clone();
        let _ = sender.send(msg).await;
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        self.sender.close_channel();
        log!("Batch dropped, channel closed");
    }
}

struct BatchStorage {
    messages: VecDeque<String>,
}

impl BatchStorage {
    fn new() -> BatchStorage {
        BatchStorage {
            messages: VecDeque::with_capacity(MAX_LENGTH),
        }
    }

    fn push_back(&mut self, s: String) {
        self.messages.push_back(s);
    }

    fn get_messages(&mut self) -> Vec<String> {
        self.messages.drain(..).collect()
    }

    fn len(&self) -> usize {
        self.messages.len()
    }
}

#[tokio::main]
async fn main() {
    let mut batch = Batch::new(Producer {});
    log!("Started");
        for i in 0..MAX_LENGTH - 1 {
        batch.push(format!("msg-{i}")).await;
    }
    tokio::time::sleep(Duration::from_millis(1100)).await;
    for i in 0..MAX_LENGTH {
        tokio::time::sleep(Duration::from_millis(200)).await;
        batch.push(format!("msg-{i}")).await;
    }
    tokio::time::sleep(Duration::from_millis(10)).await; // wait for the flush output
    batch.push("last-msg".to_string()).await;
    log!("Last message pushed");
    tokio::time::sleep(Duration::from_millis(1100)).await;
    batch.push("message-won't-send".to_string()).await;
}
