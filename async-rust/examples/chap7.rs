use futures::join;
use std::time::Instant;

async fn fetch_url(url: &str) -> String {
    println!("Fetching {}", url);
    #[cfg(feature = "smol")]
    smol::Timer::after(std::time::Duration::from_secs(5)).await;
    #[cfg(feature = "default")]
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    #[cfg(feature = "async-std")]
    async_std::task::sleep(std::time::Duration::from_secs(5)).await;
    println!("Fetched successfully {}", url);
    String::new()
}

async fn read_file(path: &str) -> Vec<u8> {
    println!("Reading {}", path);
    #[cfg(feature = "smol")]
    smol::Timer::after(std::time::Duration::from_secs(5)).await;
    #[cfg(feature = "default")]
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    #[cfg(feature = "async-std")]
    async_std::task::sleep(std::time::Duration::from_secs(5)).await;
    println!("Completed reading file {}", path);
    Vec::new()
}

#[cfg(feature = "smol")]
fn main() {
    println!("Start time: {:?}", Instant::now());
    smol::block_on(async {
        join!(
            read_file("/home/example"),
            fetch_url("https://example.com/")
        );
    });
    println!("End time: {:?}", Instant::now());
}

#[cfg(feature = "default")]
#[tokio::main]
async fn main() {
    println!("Start time: {:?}", Instant::now());
    tokio::join! {
        read_file("/home/example"),
        fetch_url("https://example.com/")
    };
    println!("End time: {:?}", Instant::now());
}

#[cfg(feature = "async-std")]
#[async_std::main]
async fn main() {
    println!("Start time: {:?}", Instant::now());
    join!(
        read_file("/home/example"),
        fetch_url("https://example.com/")
    );
    println!("End time: {:?}", Instant::now());
}
