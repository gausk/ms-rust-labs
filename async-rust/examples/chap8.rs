use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

async fn cancellation_example() {
    let handle = tokio::spawn(async {
        println!("Start task");
        sleep(Duration::from_secs(5)).await;
        println!("End of task");
        "Completed".to_string()
    });

    sleep(Duration::from_secs(1)).await;

    // Cancel the task by dropping the handle? NO — task keeps running!
    //drop(handle);

    handle.abort();

    sleep(Duration::from_secs(10)).await;

    // Awaiting an aborted task return JoinErr
    match handle.await {
        Ok(val) => println!("Got {}", val),
        Err(e) if e.is_cancelled() => println!("Task was cancelled"),
        Err(e) => println!("Task panicked: {}", e),
    }
}

async fn run_with_limit<F, Fut, T>(tasks: Vec<F>, limit: usize) -> Vec<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let semaphore = Arc::new(Semaphore::new(limit));
    let mut handles = Vec::new();
    for task in tasks {
        let permit = Arc::clone(&semaphore);
        let handle = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            task().await
        });
        handles.push(handle);
    }
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

#[tokio::main]
async fn main() {
    cancellation_example().await;
}
