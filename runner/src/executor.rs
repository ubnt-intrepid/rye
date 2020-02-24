use crate::report::Outcome;
use futures::{
    executor::{LocalSpawner, ThreadPool},
    future::{BoxFuture, Future},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use rye::{
    executor::{AsyncTest, BlockingTest, LocalAsyncTest, TestExecutor},
    TestResult,
};
use std::{io, panic::AssertUnwindSafe};

pub struct DefaultTestExecutor {
    pool: ThreadPool,
    local_spawner: LocalSpawner,
}

impl DefaultTestExecutor {
    pub fn new(local_spawner: LocalSpawner) -> io::Result<Self> {
        Ok(Self {
            pool: ThreadPool::new()?,
            local_spawner,
        })
    }
}

impl TestExecutor for DefaultTestExecutor {
    type Handle = BoxFuture<'static, Outcome>;

    fn execute(&mut self, mut test: AsyncTest) -> Self::Handle {
        let handle = self
            .pool
            .spawn_with_handle(async move { run_test(test.run()).await });
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => {
                    Outcome::failed().error_message(format!("unknown error: {}", spawn_err))
                }
            }
        })
    }

    fn execute_local(&mut self, mut test: LocalAsyncTest) -> Self::Handle {
        let handle = self
            .local_spawner
            .spawn_local_with_handle(async move { run_test(test.run()).await });
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => {
                    Outcome::failed().error_message(format!("unknown error: {}", spawn_err))
                }
            }
        })
    }

    fn execute_blocking(&mut self, mut test: BlockingTest) -> Self::Handle {
        let (tx, rx) = futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let res = maybe_unwind(AssertUnwindSafe(|| test.run()));
            let _ = tx.send(make_outcome(res));
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|rx_err| {
                Outcome::failed().error_message(format!("unknown error: {}", rx_err))
            })
        })
    }
}

async fn run_test<Fut>(fut: Fut) -> Outcome
where
    Fut: Future<Output = Box<dyn TestResult>>,
{
    let res = AssertUnwindSafe(fut).maybe_unwind().await;
    make_outcome(res)
}

fn make_outcome(res: Result<Box<dyn TestResult>, Unwind>) -> Outcome {
    let error_message = match res {
        Ok(term) if term.is_success() => None,
        Ok(term) => Some(
            term.error_message()
                .map_or("<unknown>".into(), |msg| format!("{:?}", msg)),
        ),
        Err(unwind) => Some(unwind.to_string()),
    };

    match error_message {
        Some(msg) => Outcome::failed().error_message(msg),
        None => Outcome::passed(),
    }
}
