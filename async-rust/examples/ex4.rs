use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::main;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

unsafe impl<T> Send for AsyncMutex<T> where T: Send {}
unsafe impl<T> Sync for AsyncMutex<T> where T: Send {}

unsafe impl<T> Sync for AsyncMutexGuard<'_, T> where T: Send {}

struct AsyncMutex<T> {
    data: UnsafeCell<T>,
    semaphore: Arc<Semaphore>,
}

struct AsyncMutexGuard<'a, T> {
    lock: &'a AsyncMutex<T>,
    _permit: OwnedSemaphorePermit,
}

impl<T> AsyncMutex<T> {
    fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    async fn lock(&self) -> AsyncMutexGuard<'_, T> {
        let _permit = self.semaphore.clone().acquire_owned().await.unwrap();
        AsyncMutexGuard {
            lock: self,
            _permit,
        }
    }
}

impl<T> Deref for AsyncMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: we only hold the lock
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for AsyncMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we only hold the lock
        unsafe { &mut *self.lock.data.get() }
    }
}

#[main]
async fn main() {
    let count = Arc::new(AsyncMutex::new(0));

    for i in 0..5 {
        let my_count = Arc::clone(&count);
        tokio::spawn(async move {
            for j in 0..10 {
                let mut lock = my_count.lock().await;
                *lock += 1;
                println!("{} {} {}", i, j, *lock);
            }
        });
    }

    loop {
        if *count.lock().await >= 50 {
            break;
        }
    }
    println!("Count hit 50.");
}
