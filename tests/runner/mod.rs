use futures::{
    channel::oneshot,
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{Future, FutureExt as _, RemoteHandle},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use rye::{
    reporter::{ConsoleReporter, TestCaseSummary},
    Args, Session, TestCase, TestExecutor,
};
use std::{io, sync::Arc, thread};

pub(crate) fn run_tests(tests: &[&dyn TestCase]) {
    rye::install();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());
    let mut session = Session::new(&args);

    let mut local_pool = LocalPool::new();
    let mut runner = FuturesTestRunner::new(local_pool.spawner()).unwrap();

    let reporter = Arc::new(ConsoleReporter::new(&args));
    let st = local_pool.run_until(session.run(tests, &mut runner, &reporter));

    st.exit();
}

struct FuturesTestRunner {
    thread_pool: ThreadPool,
    local_spawner: LocalSpawner,
}

impl FuturesTestRunner {
    fn new(local_spawner: LocalSpawner) -> io::Result<Self> {
        Ok(FuturesTestRunner {
            thread_pool: ThreadPool::new()?,
            local_spawner,
        })
    }
}

impl TestExecutor for FuturesTestRunner {
    type Handle = RemoteHandle<TestCaseSummary>;

    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static,
    {
        self.thread_pool.spawn_with_handle(fut).unwrap()
    }

    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static,
    {
        self.local_spawner.spawn_local_with_handle(fut).unwrap()
    }

    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let (remote, handle) = rx.map(|res| res.unwrap()).remote_handle();
        self.local_spawner.spawn(remote).unwrap();
        thread::spawn(move || {
            let _ = tx.send(f());
        });
        handle
    }
}
