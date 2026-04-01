use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

struct Ready42;

impl Future for Ready42 {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(42)
    }
}

struct Delay {
    completed: Arc<Mutex<bool>>,
    waker: Arc<Mutex<Option<Waker>>>,
    duration: Duration,
    started: bool,
}

impl Delay {
    fn new(duration: Duration) -> Self {
        Self {
            completed: Arc::new(Mutex::new(false)),
            waker: Arc::new(Mutex::new(None)),
            duration,
            started: false,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Delay polling at time: {:?}", Instant::now());
        if *self.completed.lock().unwrap() {
            return Poll::Ready(());
        }
        *self.waker.lock().unwrap() = Some(cx.waker().clone());

        if !self.started {
            self.started = true;

            let duration = self.duration;
            let completed = Arc::clone(&self.completed);
            let waker = Arc::clone(&self.waker);

            std::thread::spawn(move || {
                thread::sleep(duration);
                *completed.lock().unwrap() = true;
                if let Some(wk) = waker.lock().unwrap().take() {
                    wk.wake();
                }
            });
        }

        Poll::Pending
    }
}

struct CountDownFuture {
    countdown: Arc<Mutex<u32>>,
}

impl CountDownFuture {
    fn new(countdown: u32) -> Self {
        Self {
            countdown: Arc::new(Mutex::new(countdown)),
        }
    }
}

impl Future for CountDownFuture {
    type Output = &'static str;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut current_val = self.countdown.lock().unwrap();
        if *current_val == 0 {
            Poll::Ready("Liftoff!")
        } else {
            println!("CountDown: {}", *current_val);
            *current_val -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    let ready = Ready42;
    let output = ready.await;
    println!("ready42: {}", output);

    let delay = Delay::new(Duration::from_millis(1000));
    delay.await;

    let counter = CountDownFuture::new(10);
    let output = counter.await;
    println!("CountDown: {}", output);
}
