use std::pin::Pin;
use std::task::{Context, Poll};

async fn http_get(url: &str) -> String {
    String::new()
}

struct HttpGetFuture;

async fn fetch_two_pages() -> String {
    let page1 = http_get("https://example.com/a").await;
    let page2 = http_get("https://example.com/b").await;
    format!("{}\n{}", page1, page2)
}

enum FetchTwoPageStateMachine {
    // State 0: About to call http_get for page1
    Start,

    // State 1: Waiting for page1, holding the future
    WaitingPage1 {
        fut1: HttpGetFuture,
    },
    // state 2: Got page1, waiting for page2
    WaitingPage2 {
        page1: String,
        fut2: HttpGetFuture,
    },
    // Terminal state
    Completed,
}
/*
impl Future for FetchTwoPageStateMachine {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().get_mut() {
                Self::Start => {
                    let fut1 = http_get("https://example.com/a");
                    *self.as_mut().get_mut() = Self::WaitingPage1 { fut1 };
                }
                Self::WaitingPage1 { fut1 } => {
                    let page1 = match Pin::new(fut1).poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(result) => v,
                    };
                    let fut2 = http_get("https://example.com/b");
                    *self.as_mut().get_mut() = Self::WaitingPage2 { page1, fut2 };
                },
                Self::WaitingPage2 { page1, fut2 } => {
                    let page2 = match Pin::new(fut2).poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(v) => v,
                    };
                    let result = format!("{}\n{}", page1, page2);
                    *self.as_mut().get_mut() = Self::Completed;
                    return Poll::Ready(result);
                },
                Self::Completed => {
                    panic!("future polled after completion");
                }
            }
        }
    }
}

async fn pipeline(url: &str) -> Result<usize, Error> {
    let response = fetch(url).await?;
    let body = response.text().await?;
    let parsed = parse(body).await?;
    Ok(parsed.len())
}

enum PipelineStateMachine<'a> {
    // State 0: About to call fetch,
    Start { url: &'a str },
    // State 1, waiting for fetch, holding the future,
    Fetch {
        url: &'a str,
        f1: FetchFuture,
    },
    // state 2: Got response, waiting for body
    Response {
        response: Response,
        f2: ResponseBodyFuture,
    },
    // state 3: Got body, waiting for parsed body
    Parse { f3: ParseFuture, body: String },

    // Terminal state
    Completed
}

*/

fn main() {}