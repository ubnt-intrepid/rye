use crate::{
    context::TestContext,
    outcome::Outcome,
    test_case::{TestCase, TestFn},
};
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{
    executor::ThreadPool,
    future::{BoxFuture, Future},
    task::SpawnExt as _,
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use std::{io, panic::AssertUnwindSafe};

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

pub(crate) fn start_test<E: ?Sized>(test_case: &TestCase, executor: &mut E) -> E::Handle
where
    E: TestExecutor,
{
    let desc = test_case.desc.clone();
    match test_case.test_fn {
        TestFn::SyncTest(f) => executor.execute_blocking(move || {
            let res = expected(|| {
                maybe_unwind(AssertUnwindSafe(|| {
                    if desc.leaf_sections.is_empty() {
                        TestContext::new(&desc, None).scope(&f);
                    } else {
                        for &section in desc.leaf_sections {
                            TestContext::new(&desc, Some(section)).scope(&f);
                        }
                    }
                }))
            });
            make_outcome(res)
        }),
        TestFn::AsyncTest(f) => executor.execute(async move {
            let res = AssertUnwindSafe(async move {
                if desc.leaf_sections.is_empty() {
                    TestContext::new(&desc, None).scope_async(f()).await;
                } else {
                    for &section in desc.leaf_sections {
                        TestContext::new(&desc, Some(section))
                            .scope_async(f())
                            .await;
                    }
                }
            })
            .maybe_unwind()
            .expected()
            .await;
            make_outcome(res)
        }),
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
