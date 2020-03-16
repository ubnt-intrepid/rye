use futures::{
    channel::oneshot,
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{Future, FutureExt as _, RemoteHandle},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use rye::{
    cli::{Args, Session},
    reporter::{ConsoleReporter, TestCaseSummary},
    runner::{AsyncTest, BlockingTest, LocalAsyncTest, TestRunner},
    test::TestSet,
};
use std::{io, sync::Arc, thread};

pub(crate) fn run_tests(tests: &[&dyn TestSet]) {
    rye::cli::install();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());
    let mut session = Session::new(&args);

    let mut runner = FuturesTestRunner::new().unwrap();
    let reporter = Arc::new(ConsoleReporter::new(&args));
    let st = session.run(tests, &mut runner, &reporter);

    st.exit();
}

struct FuturesTestRunner {
    thread_pool: ThreadPool,
    local_pool: LocalPool,
    local_spawner: LocalSpawner,
}

impl FuturesTestRunner {
    fn new() -> io::Result<Self> {
        let local_pool = LocalPool::new();
        let local_spawner = local_pool.spawner();
        Ok(FuturesTestRunner {
            thread_pool: ThreadPool::new()?,
            local_pool,
            local_spawner,
        })
    }
}

impl TestRunner for FuturesTestRunner {
    type Handle = RemoteHandle<TestCaseSummary>;

    fn spawn(&mut self, mut test: AsyncTest) -> Self::Handle {
        self.thread_pool
            .spawn_with_handle(async move { test.run().await })
            .unwrap()
    }

    fn spawn_local(&mut self, mut test: LocalAsyncTest) -> Self::Handle {
        self.local_spawner
            .spawn_local_with_handle(async move { test.run().await })
            .unwrap()
    }

    fn spawn_blocking(&mut self, mut test: BlockingTest) -> Self::Handle {
        let (tx, rx) = oneshot::channel();
        let (remote, handle) = rx.map(|res| res.unwrap()).remote_handle();
        self.local_spawner.spawn(remote).unwrap();
        thread::spawn(move || {
            let is_success = test.run();
            let _ = tx.send(is_success);
        });
        handle
    }

    fn run<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()>,
    {
        self.local_pool.run_until(fut)
    }
}
