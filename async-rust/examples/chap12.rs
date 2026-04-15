// 1. Offload blocking task to dedicated thread pool using spawn_blocking
// 2. use tokio sleep instead of thread blocking sleep
// 3. Holding MutexGuard Across .await, if possible avoid it,
//    if required use async one, atleast the waiting threads won't block the thread
//    a. Use tokio::sync::Mutex if the guard crosses any .await
//    b. Add #[tracing::instrument] to async functions for span tracking
//    c. Run tokio-console in staging to catch hung tasks early
//    d. Add health check endpoints that verify task responsiveness

// 4. Cancellation Hazards
/*
async fn transfer(from: &Account, to: &Account, amount: u64) {
    from.debit(amount).await;  // If cancelled HERE...
    to.credit(amount).await;   // ...money vanishes!
}
*/

// 5. Rust drop trait is not async

use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

struct DbConnection {
    connection: Option<String>,
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        // ❌ Can't do this — drop() is sync!
        // self.connection.shutdown().await;

        // Workaround 1: Spawn a cleanup task (fire-and-forget)
        let conn = self.connection.take();
        tokio::spawn(async move {
            // let _ = conn.shutdown().await;
        });

        // ✅ Workaround 2: Use a synchronous close
        // self.connection.blocking_close();
    }
}

// 6. select! Fairness and Starvation
// tokio select is fair and pick at random if both ready.

// As fast is always ready, so when still slow is ready, fast can be chosen at random.
async fn unfair_to_slow(mut fast: Receiver<String>, mut slow: Receiver<String>) {
    tokio::select! {
        Some(v) = slow.recv() => println!("slow: {}", v),
        Some(v) = fast.recv() => println!("fast: {}", v),
    }
}

// Better to be biased toward slow, using biased in slow
async fn priortize_slow_when_ready(mut fast: Receiver<String>, mut slow: Receiver<String>) {
    tokio::select! {
        biased;
        Some(v) = slow.recv() => println!("slow: {}", v),
        Some(v) = fast.recv() => println!("fast: {}", v),
    }
}

// 7. Accidental Sequential Execution
async fn fetch_url(url: &str) {
    sleep(Duration::from_secs(1)).await;
    println!("fetching {}", url);
}

async fn sequential() {
    fetch_url("url_a").await;
    fetch_url("url_b").await;
}

// Better to use tokio::join! to do this in 1 seconds instead of two
async fn concurrent() {
    tokio::join!(fetch_url("url_a"), fetch_url("url_b"),);
}

// Use tokio-console and #[tracing::instrument] for debugging async code

fn main() {}
