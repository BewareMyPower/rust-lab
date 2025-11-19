use std::sync::Arc;

use mea::{latch::Latch, mutex::Mutex};

#[tokio::main]
async fn main() {
    mutex_demo().await;
}

async fn inc(i: i32) -> i32 {
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    i + 1
}

async fn mutex_demo() {
    let value = Arc::new(Mutex::new(0));
    let count = 3;
    let latch = Arc::new(Latch::new(count));
    for i in 0..count {
        let task_name = format!("task{}", i);
        let value_clone = value.clone();
        let latch_clone = latch.clone();
        tokio::spawn(async move {
            let mut i = value_clone.lock().await;
            for _ in 0..5 {
                *i = inc(*i).await;
                println!("{} | intermediate i = {}", task_name, *i);
            }
            drop(i);
            latch_clone.count_down();
        });
    }
    latch.wait().await;
    println!("Final value: {}", *value.lock().await);
}
