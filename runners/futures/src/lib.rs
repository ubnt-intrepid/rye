use futures::{
    channel::oneshot,
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{Future, FutureExt as _, RemoteHandle},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use rye::{reporter::TestCaseSummary, TestCase, TestExecutor, TestRunner};
use std::{io, thread};

pub fn runner(tests: &[&dyn TestCase]) {
    let mut runner = TestRunner::new();

    let mut local_pool = LocalPool::new();
    let mut executor = FuturesTestRunner::new(local_pool.spawner()).unwrap();
    local_pool
        .run_until(runner.run(tests, &mut executor))
        .unwrap();
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
