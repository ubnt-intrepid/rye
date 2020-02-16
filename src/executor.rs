use crate::outcome::Outcome;
use futures::{
    executor::ThreadPool,
    future::{BoxFuture, Future},
    task::SpawnExt as _,
};
use std::io;

pub trait TestExecutor {
    type Handle: Future<Output = Outcome>;

    fn execute<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = Outcome> + Send + 'static;
    fn execute_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> Outcome + Send + 'static;
}

pub struct DefaultTestExecutor {
    pool: ThreadPool,
}

impl DefaultTestExecutor {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            pool: ThreadPool::new()?,
        })
    }
}

impl TestExecutor for DefaultTestExecutor {
    type Handle = BoxFuture<'static, Outcome>;

    fn execute<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = Outcome> + Send + 'static,
    {
        let handle = self.pool.spawn_with_handle(fut);
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => {
                    Outcome::failed().error_message(format!("unknown error: {}", spawn_err))
                }
            }
        })
    }

    fn execute_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> Outcome + Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|rx_err| {
                Outcome::failed().error_message(format!("unknown error: {}", rx_err))
            })
        })
    }
}
