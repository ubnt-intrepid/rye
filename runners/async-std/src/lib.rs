use async_std::task;
use futures::future::{Future, FutureExt as _, LocalBoxFuture};
use rye::{report::TestCaseSummary, runner::TestRunner, TestCase, TestExecutor};

pub fn runner(tests: &[&dyn TestCase]) {
    let mut runner = TestRunner::new();

    task::block_on(async {
        let mut executor = AsyncStdTestRunner { _p: () };

        runner.run(tests, &mut executor).await.unwrap();
    })
}

struct AsyncStdTestRunner {
    _p: (),
}

impl TestExecutor for AsyncStdTestRunner {
    type Handle = LocalBoxFuture<'static, TestCaseSummary>;

    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static,
    {
        task::spawn(fut).boxed()
    }

    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static,
    {
        Box::pin(fut)
    }

    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static,
    {
        task::spawn_blocking(f).boxed()
    }
}
