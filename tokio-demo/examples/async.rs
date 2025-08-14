use std::{
    pin::Pin,
    sync::{Arc, Mutex, mpsc},
    task::{Context, Poll},
    thread,
    time::{Duration, Instant},
};

use futures::task::{self, ArcWake};

macro_rules! log {
    ($($arg:tt)*) => {
        let now = chrono::Local::now();
        print!("{} - {} - ", now.format("%Y-%m-%d %H:%M:%S%.3f"), thread::current().name().unwrap_or("unknown"));
        println!($($arg)*);
    };
}

fn main() {
    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(async {
        log!("Before delay");
        let when = Instant::now() + Duration::from_millis(1000);
        let future = Delay { when };
        let out = future.await;
        log!("After delay");
        assert_eq!(out, "done");
    });
    mini_tokio.run();
    // TODO: stop the runtime
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        log!("poll() is called");
        if Instant::now() >= self.when {
            log!("task is done");
            Poll::Ready("done")
        } else {
            let waker = cx.waker().clone();
            let when = self.when;

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                log!("waker.wake()");
                waker.wake();
            });

            Poll::Pending
        }
    }
}

// It wraps the user-provided future and its poll state.
// This struct itself is also a future that implements the poll method to poll the wrapped future
// and update the state to the state.
struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}

impl TaskFuture {
    fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        // polling a future which has already returned `Ready` is not allowed, so we need to check
        // if it's pending.
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
        self.poll
    }
}

struct Task {
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    fn poll(self: &Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();

        let _ = task_future.poll(&mut cx);
    }

    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Create a wrapped task future and send it to the channel, then the task future will be
        // received in `MiniTokio::run` and polled.
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // When the task's future poll call wake() or wake_by_ref() on the context, send the task to the channel again so that it can be polled again in `MiniTokio::run`.
        log!("schedule the task again");
        arc_self.schedule();
    }
}

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = mpsc::channel();

        MiniTokio { scheduled, sender }
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }
}
