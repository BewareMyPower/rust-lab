use std::{collections::VecDeque, sync::Arc, thread, time::Duration, usize};

use futures::{channel::mpsc, lock::Mutex, SinkExt, StreamExt};

const MAX_LENGTH: usize = 5;
const BATCH_TIMEOUT: Duration = Duration::from_secs(1);

macro_rules! log {
    ($($arg:tt)*) => {
        let now = chrono::Local::now();
        print!("{} - {} - ", now.format("%Y-%m-%d %H:%M:%S%.3f"), thread::current().name().unwrap_or("unknown"));
        println!($($arg)*);
    };
}

struct Batch {
    storage: Arc<Mutex<BatchStorage>>,
    sender: mpsc::Sender<Vec<String>>,
}

impl Batch {
    fn new() -> Batch {
        let (sender, mut receiver) = mpsc::channel::<Vec<String>>(1);
        tokio::spawn(async move {
            loop {
                let result = receiver.next().await;
                match result {
                    Some(messages) => {
                        log!("Received batch: {:?}", messages);
                    }
                    None => {
                        log!("Receiver channel closed");
                        break;
                    }
                }
            }
        });
        Batch {
            storage: Arc::new(Mutex::new(BatchStorage::new())),
            sender: sender,
        }
    }

    async fn push(&mut self, s: String) {
        let mut storage = self.storage.lock().await;
        let mut sender = self.sender.clone();
        if storage.len() >= MAX_LENGTH {
            let _ = sender.send(storage.get_messages()).await;
        } else if storage.len() == 1 {
            let storage_clone = self.storage.clone();
            log!("Started timer");
            tokio::spawn(async move {
                tokio::time::sleep(BATCH_TIMEOUT).await;
                let mut storage = storage_clone.lock().await;
                if storage.len() > 0 {
                    let _ = sender.send(storage.get_messages()).await.unwrap();
                }
            });
        }
        storage.push_back(s);
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
    let mut batch = Batch::new();
    log!("Started");
    for i in 0..MAX_LENGTH - 1 {
        batch.push(format!("msg-{i}")).await;
    }
    tokio::time::sleep(Duration::from_millis(1200)).await;
    for i in 0..MAX_LENGTH {
        tokio::time::sleep(Duration::from_millis(200)).await;
        batch.push(format!("msg-{i}")).await;
    }
    batch.push("last-msg".to_string()).await;
    log!("Last message pushed");
    tokio::time::sleep(Duration::from_millis(1000)).await;
}
