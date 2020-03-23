use futures::{
    future::Future,
    task::{self, Poll},
};
use pin_project_lite::pin_project;
use rye::{reporter::TestCaseSummary, TestCase, TestExecutor, TestRunner};
use std::pin::Pin;
use tokio::{
    runtime::{self, Handle},
    task::{JoinHandle as RawJoinHandle, LocalSet},
};

pub fn runner(tests: &[&dyn TestCase]) {
    let mut runner = TestRunner::new();

    let mut rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();

    let local_set = LocalSet::new();

    let mut executor = TokioTestRunner {
        handle: rt.handle().clone(),
        local_set: &local_set,
    };

    local_set
        .block_on(&mut rt, runner.run(tests, &mut executor))
        .unwrap();
}

struct TokioTestRunner<'a> {
    handle: Handle,
    local_set: &'a LocalSet,
}

impl TestExecutor for TokioTestRunner<'_> {
    type Handle = JoinHandle;

    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static,
    {
        JoinHandle {
            raw: self.handle.spawn(fut),
        }
    }

    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static,
    {
        JoinHandle {
            raw: self.local_set.spawn_local(fut),
        }
    }

    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static,
    {
        JoinHandle {
            raw: tokio::task::spawn_blocking(f),
        }
    }
}

pin_project! {
    struct JoinHandle {
        #[pin]
        raw: RawJoinHandle<TestCaseSummary>,
    }
}

impl Future for JoinHandle {
    type Output = TestCaseSummary;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        me.raw.poll(cx).map(|res| {
            res.unwrap() // FIXME: report join error
        })
    }
}
