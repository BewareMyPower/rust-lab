// Copyright 2025 Yunze Xu
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod thread_pool;
use crate::thread_pool::ThreadPool;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
    time::Duration,
};

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        let now = chrono::Local::now();
        print!("{} - {} - ", now.format("%Y-%m-%d %H:%M:%S%.3f"), thread::current().name().unwrap_or("unknown"));
        println!($($arg)*);
    };
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let url = match args.len() {
        1 => "127.0.0.1:7878",
        _ => args[1].as_str(),
    };
    log!("Listening on {}", &url);

    let listener = TcpListener::bind(url).unwrap();
    listener.set_nonblocking(true).unwrap();
    let running = Arc::new(AtomicBool::new(true));

    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        running_clone.store(false, std::sync::atomic::Ordering::Release);
        log!("The server is shutting down and will exit after accepting another connection");
    })
    .unwrap();

    let pool = ThreadPool::new(4);
    let pool = Arc::new(Mutex::new(pool));
    let pool_clone = pool.clone();
    thread::Builder::new()
        .name(String::from("acceptor"))
        .spawn(move || {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        log!("Accepted connection from {}", addr);
                        pool_clone
                            .lock()
                            .unwrap()
                            .execute(move || handle_connection(stream));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if !running.load(std::sync::atomic::Ordering::Acquire) {
                            log!("Server has been shut down, exiting");
                            return;
                        }
                        // TODO: avoid busy wait
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        log!("Failed to accept connection: {}", e);
                        return;
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
    pool.lock().unwrap().shutdown();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);
    let mut line = String::new();
    buf_reader.read_line(&mut line).unwrap();

    let (status_line, filename) = match line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_millis(500));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let content = std::fs::read_to_string(filename).unwrap();
    let response = format!(
        "{status_line}\r\nContent-Length: {}\r\n\r\n{content}",
        content.len()
    );
    stream.write_all(response.as_bytes()).unwrap();
}
