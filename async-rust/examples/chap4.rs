// 1. Pin<P> is a wrapper that prevents the pointee from being moved —
//    essential for self-referential state machines
// 2. Box::pin() is the safe, easy default for pinning futures on the heap
// 3. tokio::pin!() pins on the stack — you can move the Pin<&mut> wrapper, but the underlying future stays put
// 4. Unpin is an auto-trait opt-out: types that implement Unpin can be moved even
//    when pinned (most types are Unpin; async blocks are not)

/*
// 1. poll() signature — all futures are polled through Pin
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Output>;

// 2. Box::pin() — heap-allocate and pin a future
let future: Pin<Box<dyn Future<Output = i32>>> = Box::pin(async { 42 });

// 3. tokio::pin!() — pin a future on the stack
tokio::pin!(my_future);
// Now my_future: Pin<&mut impl Future>
 */
fn main() {
    use std::pin::Pin;

    let mut data = String::from("hello");

    // Pin it — now it can't be moved
    let pinned: Pin<&mut String> = Pin::new(&mut data);

    // Can still use it:
    println!("{}", pinned.as_ref().get_ref());

    let mutable: &mut String = Pin::into_inner(pinned); // Only if String: Unpin
}
