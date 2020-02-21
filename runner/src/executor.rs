use crate::report::Outcome;
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{executor::ThreadPool, future::BoxFuture, task::SpawnExt as _};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use rye::executor::{AsyncTestBody, TestBody, TestExecutor};
use std::{io, panic::AssertUnwindSafe};

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

    fn execute(&mut self, mut test: TestBody) -> Self::Handle {
        let (tx, rx) = futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let res = expected(|| maybe_unwind(AssertUnwindSafe(|| test.run())));
            let _ = tx.send(make_outcome(res));
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|rx_err| {
                Outcome::failed().error_message(format!("unknown error: {}", rx_err))
            })
        })
    }

    fn execute_async(&mut self, mut test: AsyncTestBody) -> Self::Handle {
        let handle = self.pool.spawn_with_handle(async move {
            let res = AssertUnwindSafe(test.run()).maybe_unwind().expected().await;
            make_outcome(res)
        });
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => {
                    Outcome::failed().error_message(format!("unknown error: {}", spawn_err))
                }
            }
        })
    }
}

fn make_outcome(res: (Result<(), Unwind>, Option<Disappoints>)) -> Outcome {
    match res {
        (Ok(()), None) => Outcome::passed(),
        (Ok(()), Some(disappoints)) => Outcome::failed().error_message(disappoints.to_string()),
        (Err(unwind), disappoints) => {
            use std::fmt::Write as _;
            let mut msg = String::new();
            let _ = writeln!(&mut msg, "{}", unwind);
            if let Some(disappoints) = disappoints {
                let _ = writeln!(&mut msg, "{}", disappoints);
            }
            Outcome::failed().error_message(msg)
        }
    }
}
