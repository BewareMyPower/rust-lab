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

use crate::log;
use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{Sender, channel},
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    active_id: usize,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let mut workers: Vec<Worker> = vec![];
        for i in 0..num_workers {
            let worker = Worker::new(i);
            log!("Worker {} created", i);
            workers.push(worker);
        }
        ThreadPool {
            workers: workers,
            active_id: 0,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let index = self.active_id % self.workers.len();
        self.active_id = index + 1;
        self.workers[index].sender.send(Box::new(f)).unwrap();
    }

    pub fn shutdown(&mut self) {
        let workers = self.workers.drain(..).collect::<Vec<_>>();
        for worker in workers {
            worker
                .running
                .store(false, std::sync::atomic::Ordering::Release);
            worker.sender.send(Box::new(|| {})).unwrap();
            worker.handle.join().unwrap();
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    sender: Sender<Job>,
    handle: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl Worker {
    fn new(id: usize) -> Self {
        let (sender, receiver) = channel::<Job>();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let handle = thread::Builder::new()
            .name(format!("worker-{}", id))
            .spawn(move || {
                loop {
                    match receiver.recv() {
                        Ok(job) => {
                            if !running_clone.load(std::sync::atomic::Ordering::Acquire) {
                                log!("Worker {} is shutting down", id);
                                return;
                            }
                            log!("Worker {} received a job", id);
                            job();
                        }
                        Err(_) => {
                            log!("Worker {} disconnected; exiting.", id);
                            break;
                        }
                    }
                }
            })
            .unwrap();
        Worker {
            sender,
            handle,
            running,
        }
    }
}
