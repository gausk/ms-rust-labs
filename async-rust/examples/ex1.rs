/*
Exercise 1: Async Echo Server
Build a TCP echo server that handles multiple clients concurrently.

Requirements:

Listen on 127.0.0.1:8080
Accept connections and echo back each line
Handle client disconnections gracefully
Print a log when clients connect/disconnect
 */
use futures::future::join_all;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::time::sleep;
use tokio::{select, signal};

async fn process_accept(
    mut shutdown_rx: watch::Receiver<bool>,
    mut conn: TcpStream,
) -> tokio::io::Result<()> {
    println!("Accepted new connection");
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        select! {
             len = reader.read_line(&mut line) => {
                match len {
                    Ok(n) if n == 0 => {
                        println!("Connection closed");
                        return Ok(());
                    }
                    Ok(n) => {
                        println!("Received {} bytes", n);
                        writer.write_all(line.as_bytes()).await?;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            },
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn run_server(mut shutdown: watch::Receiver<bool>) -> tokio::io::Result<()> {
    println!("Server starting...");
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        select! {
            conn = listener.accept() => {
                let shutdown = shutdown.clone();
                tokio::spawn(process_accept(shutdown, conn?.0));
            },
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    println!("stop accepting new connections");
                    break;
                }
            }
        }
    }
    Ok(())
}

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

#[tokio::main]
async fn main() {
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
