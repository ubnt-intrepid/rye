use futures::{
    channel::oneshot,
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{Future, FutureExt as _, RemoteHandle},
    task::{self, LocalSpawnExt as _, Poll, SpawnExt as _},
};
use rye::{
    cli::{Args, Session},
    reporter::ConsoleReporter,
    runner::{AsyncTest, BlockingTest, LocalAsyncTest, Summary, TestCaseResult, TestRunner},
    test::Registration,
};
use std::{io, pin::Pin, thread};

pub(crate) fn run_tests(tests: &[&dyn Registration]) {
    rye::cli::install();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());
    let mut session = Session::new(&args);

    let mut runner = FuturesTestRunner::new().unwrap();
    let mut reporter = ConsoleReporter::new(&args);
    let st = session.run(tests, &mut runner, &mut reporter);

    st.exit();
}

#[pin_project::pin_project]
struct InFlight {
    handle: RemoteHandle<TestCaseResult>,
    result: Option<TestCaseResult>,
}

impl Future for InFlight {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let result = futures::ready!(me.handle.poll_unpin(cx));
        me.result.replace(result);
        Poll::Ready(())
    }
}

struct FuturesTestRunner {
    thread_pool: ThreadPool,
    local_pool: LocalPool,
    local_spawner: LocalSpawner,
    in_flights: Vec<InFlight>,
}

impl FuturesTestRunner {
    fn new() -> io::Result<Self> {
        let local_pool = LocalPool::new();
        let local_spawner = local_pool.spawner();
        Ok(FuturesTestRunner {
            thread_pool: ThreadPool::new()?,
            local_pool,
            local_spawner,
            in_flights: vec![],
        })
    }
}

impl TestRunner for FuturesTestRunner {
    fn spawn(&mut self, mut test: AsyncTest) {
        let handle = self
            .thread_pool
            .spawn_with_handle(async move { test.run().await })
            .unwrap();
        self.in_flights.push(InFlight {
            handle,
            result: None,
        });
    }

    fn spawn_local(&mut self, mut test: LocalAsyncTest) {
        let handle = self
            .local_spawner
            .spawn_local_with_handle(async move { test.run().await })
            .unwrap();
        self.in_flights.push(InFlight {
            handle,
            result: None,
        });
    }

    fn spawn_blocking(&mut self, mut test: BlockingTest) {
        let (tx, rx) = oneshot::channel();
        let (remote, handle) = rx.map(|res| res.unwrap()).remote_handle();
        self.local_spawner.spawn(remote).unwrap();
        thread::spawn(move || {
            let is_success = test.run();
            let _ = tx.send(is_success);
        });
        self.in_flights.push(InFlight {
            handle,
            result: None,
        });
    }

    fn run(&mut self) -> Summary {
        let in_flights = &mut self.in_flights;
        let mut summary = Summary::default();
        self.local_pool.run_until(async {
            futures::future::join_all(in_flights.iter_mut()).await;
            for in_flight in in_flights.drain(..) {
                let result = in_flight.result.unwrap();
                summary.append(result);
            }
        });
        summary
    }
}
