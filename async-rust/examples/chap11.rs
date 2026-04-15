use async_stream::{stream, try_stream};
use futures::stream::{self, StreamExt};
use futures::{Stream, pin_mut};
use std::ops::Not;
use std::time::Duration;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};

#[tokio::main]
async fn main() {
    // 1. From an iterator
    let s = stream::iter(1..5);
    s.for_each(|x| async move {
        println!("{x}");
    })
    .await;

    let square: Vec<_> = stream::iter(vec![1, 2, 3]).map(|x| x * x).collect().await;
    println!("{:?}", square);

    let evens = stream::iter(0..=20)
        .filter(|x| futures::future::ready(x % 2 == 0))
        .collect::<Vec<_>>()
        .await;
    println!("{:?}", evens);

    let first_three = stream::iter(0..=20).take(3).collect::<Vec<_>>().await;
    println!("{:?}", first_three);

    let results = stream::iter(vec!["url1", "url2", "url3"])
        .map(|url| async move {
            sleep(Duration::from_secs(1)).await;
            format!("url: {}", url)
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    println!("{:?}", results);

    // 2.  From an async generator (using async_stream crate)
    let stream = stream! {
        for i in 0..=5 {
            yield i;
        }
    };

    pin_mut!(stream);
    while let Some(x) = stream.next().await {
        println!("{x}");
    }

    fn double<S: Stream<Item = u32>>(stream: S) -> impl Stream<Item = u32> {
        stream! {
            for await x in stream {
                yield x*x;
            }
        }
    }

    let ds = double(stream! { for i in 0..=5 { yield i;}});
    pin_mut!(ds);
    while let Some(x) = ds.next().await {
        println!("{x}");
    }

    // 3. Interval Stream
    let mut stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(1)));
    let start = tokio::time::Instant::now();
    for _ in 0..=3 {
        if let Some(instant) = stream.next().await {
            println!("Time since start: {:?}", instant.duration_since(start));
        }
    }

    // 4. From a channel receiver
    let (tx, rx) = tokio::sync::mpsc::channel::<&str>(10);
    let mut rx_stream = ReceiverStream::new(rx);
    for s in ["a", "b", "c", "d", "e", "f"] {
        tx.send(s).await.unwrap();
    }
    drop(tx);

    // while let Some(s) = rx.next().await {
    //     println!("{s}");
    // }

    while let Some(s) = rx_stream.next().await {
        println!("{s}");
    }

    // 5. from Unfold
    let st = stream::unfold(0u32, |state| async move {
        if state > 5 {
            None
        } else {
            let next = state + 1;
            Some((state, next))
        }
    });

    pin_mut!(st);

    while let Some(s) = st.next().await {
        println!("{s}");
    }
}

fn get_users() -> impl Stream<Item = Result<TcpStream, std::io::Error>> + 'static {
    try_stream! {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("Got a request: {:?}", addr);
            yield stream;
        }
    }
}

#[derive(Debug)]
struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

async fn stats_aggregator<S: Stream<Item = f64>>(stream: S) -> Stats {
    stream
        .fold(
            Stats {
                min: f64::MAX,
                max: f64::MIN,
                sum: 0.0,
                count: 0,
            },
            |mut acc, x| async move {
                acc.count += 1;
                acc.sum += x;
                acc.max = acc.max.max(x);
                acc.min = acc.min.min(x);
                acc
            },
        )
        .await
}

async fn io_examples() -> tokio::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    // AsyncWriteExt: write_all, write_u32, write_buf
    stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await?;

    // AsyncReadExt: read, read_exact, read_to_end, read_to_string
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;

    // AsyncBufReadExt: read_line, lines(), split()
    let file = tokio::fs::File::open("Cargo.toml").await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        println!("{line:?}");
    }
    Ok(())
}

// A length prefixed protocol: [u32: length][payload bytes]
struct FramedStream<T> {
    inner: T,
}

impl<T: AsyncRead + Unpin> FramedStream<T> {
    async fn read_frame(&mut self) -> tokio::io::Result<Vec<u8>> {
        let length = self.inner.read_u32().await?;
        let mut buf = vec![0u8; length as usize];
        self.inner.read_buf(&mut buf).await?;
        Ok(buf)
    }
}

impl<T: AsyncWrite + Unpin> FramedStream<T> {
    async fn write_frame(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        self.inner.write_u32(buf.len() as u32).await?;
        self.inner.write_all(buf).await?;
        self.inner.flush().await?;
        Ok(())
    }
}

struct LineCounter<T> {
    inner: T,
}

impl<T: AsyncBufRead + Unpin> LineCounter<T> {
    async fn line_count(&mut self) -> usize {
        let mut lines = (&mut self.inner).lines();
        let mut count = 0;
        while let Ok(Some(line)) = lines.next_line().await {
            if line.is_empty().not() {
                count += 1;
            }
        }
        count
    }
}
