use std::future::poll_fn;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// Create a no-op waker (just keep pooling)
fn noop_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(std::ptr::null(), &vtable)
}

fn block_on<F: Future>(mut future: F) -> F::Output {
    // Pin the future on the stack
    // SAFETY: `future` is never moved after this point,
    // we access through reference from here onwards
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    // SAFETY: noop_raw_waker() return a valid waker
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut context = Context::from_waker(&waker);

    // Busy loop until future completes
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                // we should ideally park the thread and wait for waker.wake()
                // but here we just spin
                std::thread::yield_now();
            }
        }
    }
}

/*
struct TaskQueue;
struct Task;

impl Task {
    fn complete(&mut self, output: ()) {
        println!("complete");
    }

    fn waker() -> Waker {
        unsafe { Waker::from_raw(noop_raw_waker()) }
    }
}

impl Future for Task {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("task complete");
        Poll::Ready(())
    }
}

impl TaskQueue {
    fn get_woken_task(&mut self) -> Option<Task> {
        Some(Task)
    }

    fn wait_for_events(&mut self) {}
}

fn executor_loop(tasks: &mut TaskQueue) {
    loop {
        // 1. poll all the task that as been woken
        while let Some(mut task) = tasks.get_woken_task() {
            match task.poll(Context::from_waker(&Task::waker())) {
                Poll::Ready(output) => task.complete(output),
                Poll::Pending => { /* task stays in queue */ }
            }
        }
        // 2. Sleep until something wakes up (epoll_wait, kevent, etc.)
        // this is where mio/polling does the heavy lifting
        tasks.wait_for_events()
    }
}
*/

// Rules for implementing poll()
// 1. Never block — return Pending immediately if not ready
// 2. Always re-register the waker — it may have changed between polls
// 3. Handle spurious wakes — check the actual condition, don't assume readiness
// 4. Don't poll after Ready — behavior is unspecified (may panic, return Pending, or repeat Ready). Only FusedFuture guarantees safe post-completion polling

struct CountDownFuture {
    countdown: u32,
}

impl CountDownFuture {
    fn new(countdown: u32) -> Self {
        Self { countdown }
    }
}

impl Future for CountDownFuture {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.countdown == 0 {
            Poll::Ready("Liftoff!")
        } else {
            println!("CountDown: {}", self.countdown);
            self.countdown -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/*
struct MySource;
struct Data;
async fn read_when_ready(source: &MySource) -> Data {
    poll_fn(|cx| source.poll_read(cx)).await
}
*/

// yield_now() voluntarily yield control to the executor
// Useful in CPU-heavy async loops to avoid starving other tasks
/*
async fn cpu_heavy_work(items: &[Item]) {
    for (i, item) in items.iter().enumerate() {
        process(item); // CPU work

        // Every 100 items, yield to let other tasks run
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
}
*/

fn main() {
    let output = block_on(CountDownFuture::new(5));
    println!("output: {}", output);

    /*
    let value = poll_fn(|cx| {
        // Do Something with cx.waker(), return Ready or Pending
        Poll::Ready(42)
    }).await;
     */
}
