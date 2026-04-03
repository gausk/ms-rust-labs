use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::borrow::Cow;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

async fn process(x: Cow<'_, str>) -> usize {
    sleep(Duration::from_secs(1));
    println!("processing: {}", x);
    x.len()
}

// tokio::spawn() require Send + 'static

// 1. FuturesUnordered — avoids 'static entirely (no spawn!)
async fn process_items(items: &[String]) {
    let futures: FuturesUnordered<_> = items
        .iter()
        .map(|item| async move { process(Cow::Borrowed(item)).await })
        .collect();

    futures
        .for_each(|result| async move {
            println!("Task completed: {}", result);
        })
        .await;
}

// 2. tokio::task::LocalSet — run !Send futures on current thread
//    Still requires 'static — solves Send, not 'static
use tokio::task::LocalSet;

// 3. tokio JoinSet — managed set of spawned tasks
//    Still requires 'static + Send — solves task *management*,
//    not the 'static problem. Useful for tracking, aborting, and
//    joining a dynamic group of tasks.
use tokio::task::JoinSet;

async fn with_joinset() {
    let mut set = JoinSet::new();

    for i in 0..10 {
        // i is Copy and moved into the closure — already 'static.
        // You'd still need Arc or clone for borrowed data.
        set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            i * 2
        });
    }

    while let Some(result) = set.join_next().await {
        println!("Task completed: {:?}", result.unwrap());
    }
}

async fn joinset_spawn(items: &[String]) {
    let mut set = JoinSet::new();
    for item in items {
        let item = item.clone();
        set.spawn(async move { process(Cow::Owned(item)).await });
    }

    while let Some(result) = set.join_next().await {
        println!("Task completed: {:?}", result.unwrap());
    }
}

#[tokio::main]
async fn main() {
    println!("start time: {:?}", SystemTime::now());
    process_items(&["abc".to_string(), "def".to_string(), "ghi".to_string()]).await;
    println!("end time: {:?}", SystemTime::now());

    let local_set = LocalSet::new();
    local_set
        .run_until(async {
            tokio::task::spawn_local(async {
                // Can use Rc, Cell, and other !Send types here
                let rc = std::rc::Rc::new(42);
                println!("{rc}");
            })
            .await
            .unwrap();
        })
        .await;

    with_joinset().await;

    println!("start time: {:?}", SystemTime::now());
    joinset_spawn(&["a".to_string(), "de".to_string(), "ghi".to_string()]).await;
    println!("end time: {:?}", SystemTime::now());
}
