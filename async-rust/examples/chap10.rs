use std::collections::HashMap;
use std::fmt::Error;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::{LocalSet, spawn_local};
use tokio::time::sleep;

// This didn't compile until Rust 1.75 (Dec 2023):
trait DataStore {
    async fn get(&self, key: &str) -> Option<String>;
    // Desugars to
    // fn get(&self, key: &str) -> impl Future<Output = Option<String>>;
}

// Why? Because async fn returns `impl Future<Output = T>`,
// and `impl Trait` in trait return position wasn't supported.

struct InMemoryDataStore {
    data: HashMap<String, String>,
}

impl InMemoryDataStore {
    fn new() -> Self {
        Self {
            data: HashMap::from_iter(vec![
                ("hello".to_string(), "world".to_string()),
                ("world".to_string(), "hello".to_string()),
            ]),
        }
    }
}

impl DataStore for InMemoryDataStore {
    async fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}

async fn lookup<S: DataStore>(store: &S, key: &str) {
    if let Some(value) = store.get(key).await {
        println!("{key}: {value}");
    }
}

/*
async fn lookup_dyn(store: &dyn DataStore, key: &str) {
    lookup(store, key).await;
}
*/

// Return a boxed future
trait DynDataStore {
    fn get_dyn<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>;
}

impl DynDataStore for InMemoryDataStore {
    fn get_dyn<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        Box::pin(async move { self.data.get(key).cloned() })
    }
}

async fn lookup_dyn(store: &dyn DynDataStore, key: &str) {
    if let Some(value) = store.get_dyn(key).await {
        println!("{key}: {value}");
    }
}

#[async_trait::async_trait]
trait AsyncDynDataStore {
    async fn get_async_dyn(&self, key: &str) -> Option<String>;
}

#[async_trait::async_trait]
impl AsyncDynDataStore for InMemoryDataStore {
    async fn get_async_dyn(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}

async fn lookup_async_dyn(store: &dyn AsyncDynDataStore, key: &str) {
    if let Some(value) = store.get_async_dyn(key).await {
        println!("{key}: {value}");
    }
}

trait Worker {
    async fn run(self);
}

struct MyWorker;

impl Worker for MyWorker {
    async fn run(self) {
        let rc = Rc::new(42);
        sleep(Duration::from_millis(1000)).await;
        println!("rc: {rc}");
    }
}

#[trait_variant::make(SendDataStoreNew: Send + Sync)]
trait DataStoreNew {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String);
}

async fn spawn_lookup<S>(store: Arc<S>)
where
    S: SendDataStoreNew + 'static,
{
    tokio::spawn(async move {
        store.get("key").await;
    });
}

async fn retry<F>(max: usize, f: F) -> Result<String, Error>
where
    F: AsyncFn() -> Result<String, Error>,
{
    for _ in 0..max {
        if let Ok(val) = f().await {
            return Ok(val);
        }
    }
    f().await
}

trait Cache {
    async fn set(&self, key: &str, value: String);
    async fn get(&self, key: &str) -> Option<String>;
}

#[derive(Default)]
struct InMemoryCache {
    data: Mutex<HashMap<String, String>>,
}

impl Cache for InMemoryCache {
    async fn set(&self, key: &str, value: String) {
        self.data.lock().await.insert(key.to_string(), value);
    }

    async fn get(&self, key: &str) -> Option<String> {
        self.data.lock().await.get(key).cloned()
    }
}

struct Redis {
    latency: Duration,
    data: Mutex<HashMap<String, String>>,
}

impl Redis {
    fn new(latency: Duration) -> Self {
        Self {
            latency,
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl Cache for Redis {
    async fn set(&self, key: &str, value: String) {
        sleep(self.latency).await;
        self.data.lock().await.insert(key.to_string(), value);
    }

    async fn get(&self, key: &str) -> Option<String> {
        sleep(self.latency).await;
        self.data.lock().await.get(key).cloned()
    }
}

async fn cache_demo<C: Cache>(cache: &C, label: &str) {
    cache.set("greeting", "Hello, async!".into()).await;
    let val = cache.get("greeting").await;
    println!("[{label}] greeting = {val:?}");
}

#[tokio::main]
async fn main() {
    let store = InMemoryDataStore::new();
    store.get("hello").await;
    store.get("world").await;
    lookup(&store, "hello").await;
    lookup_dyn(&store, "world").await;
    let local_set = LocalSet::new();
    local_set
        .run_until(async {
            let worker = MyWorker;
            spawn_local(worker.run());
        })
        .await;

    lookup_async_dyn(&store, "world").await;

    let mem = InMemoryCache::default();
    cache_demo(&mem, "memory").await;

    let redis = Redis::new(Duration::from_secs(1));
    cache_demo(&redis, "redis").await;
}
