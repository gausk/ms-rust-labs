use futures::future::join_all;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep};
use tokio::{select, signal};
use tokio_util::task::TaskTracker;

async fn main_server() {
    // Create a shutdown signal channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // run async server
    let server_handle = tokio::spawn(run_server(shutdown_rx));

    // wait for cancel signal
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    println!("received Ctrl+C, shutting down...");

    shutdown_tx.send(true).unwrap();

    match tokio::time::timeout(Duration::from_secs(5), server_handle).await {
        Ok(Ok(_)) => println!("server shutdown gracefully..."),
        Ok(Err(e)) => println!("server error: {}", e),
        Err(_) => eprintln!("server shutdown timed out..."),
    }
}

async fn process_accept(mut shutdown_rx: watch::Receiver<bool>, mut conn: TcpStream) {
    let mut data = vec![0; 1024];
    loop {
        select! {
             _ = conn.read_exact(&mut data) => {
                println!("Received data: {:?}", data);
                data.clear();
            },
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }
}

async fn run_server(mut shutdown: watch::Receiver<bool>) -> tokio::io::Result<()> {
    println!("server starting...");
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        select! {
            conn = listener.accept() => {
                let shutdown = shutdown.clone();
                tokio::spawn(process_accept(shutdown, conn?.0));
            },
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    println!("stop accepting connections");
                    break;
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct WorkItem {
    data: i32,
}

async fn backpressure() {
    let (tx, mut rx) = mpsc::channel::<WorkItem>(5);
    let producer = tokio::spawn(async move {
        for i in 0..20 {
            tx.send(WorkItem { data: i }).await.unwrap();
            println!(
                "Time now when sending item {i} at time {:?}",
                Instant::now()
            );
        }
    });

    let consumer = tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            println!("item: {}", item.data);
            sleep(Duration::from_secs(1)).await;
        }
    });

    join_all(vec![producer, consumer]).await;
}

async fn structured_concurrency() {
    let mut set: JoinSet<Result<i32, i32>> = JoinSet::new();
    for i in 0..10 {
        set.spawn(async move {
            sleep(Duration::from_secs(1)).await;
            Ok(i)
        });
    }

    let mut results = Vec::with_capacity(10);
    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok(i)) => results.push(i),
            Ok(Err(e)) => eprintln!("task failed: {:?}", e),
            Err(e) => eprintln!("task panicked: {:?}", e),
        }
    }
    println!("Result order with JoinSet: {:?}", results);
}

async fn concurrency_with_tracker() {
    let tracker = TaskTracker::new();
    for i in 0..10 {
        tracker.spawn(async move {
            sleep(Duration::from_secs(1)).await;
            println!("task completed using TaskTracker: {i}");
            Ok::<i32, i32>(i)
        });
    }

    tracker.close();
    tracker.wait().await;
    println!("Task tracker closed as all task completed");
}

async fn exponential_backoff_retry<F, Fut, T, E>(
    base_delay_ms: u64,
    max_attempts: usize,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = Duration::from_secs(base_delay_ms);

    for attempt in 1..=max_attempts {
        match f().await {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => {
                if attempt == max_attempts {
                    eprintln!("Final attempt {attempt} failed: {e}");
                    return Err(e);
                }
                eprintln!("Attempt {attempt} failed: {e}, retrying in {delay:?}");
                sleep(delay).await;
                delay *= 2;
            }
        }
    }
    unreachable!()
}

#[tokio::main]
async fn main() {
    structured_concurrency().await;
    concurrency_with_tracker().await;

    exponential_backoff_retry(10, 5, || async move {
        sleep(Duration::from_secs(1)).await;
        Ok::<_, String>(())
    })
    .await
    .unwrap();

    backpressure().await;

    let server_handle = tokio::spawn(main_server());
    let ctrlc_handle = tokio::spawn(async {
        sleep(Duration::from_secs(120)).await;

        println!("Sending Ctrl+C (SIGINT)...");
        unsafe {
            libc::raise(libc::SIGINT);
        }
    });
    join_all(vec![server_handle, ctrlc_handle]).await;
}
