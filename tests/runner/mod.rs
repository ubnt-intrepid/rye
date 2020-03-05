use futures::{
    channel::oneshot,
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{Future, FutureExt as _, RemoteHandle},
    task::{self, LocalSpawnExt as _, Poll, SpawnExt as _},
};
use rye::{
    cli::{Args, Session},
    executor::{AsyncTest, BlockingTest, LocalAsyncTest, Summary, TestCaseResult, TestExecutor},
    reporter::ConsoleReporter,
    test::Registration,
};
use std::{pin::Pin, thread};

pub(crate) fn run_tests(tests: &[&dyn Registration]) {
    rye::cli::install();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());
    let mut session = Session::new(&args);

    if let Err(err) = session.register_tests(tests) {
        eprintln!("registry error: {}", err);
        std::process::exit(101);
    }

    let local_pool = LocalPool::new();
    let local_spawner = local_pool.spawner();
    let mut executor = FuturesExecutor {
        pool: ThreadPool::new().unwrap(),
        local_pool,
        local_spawner,
        in_flights: vec![],
    };

    let mut printer = ConsoleReporter::new(&args);
    let st = session.run(&mut executor, &mut printer);
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

struct FuturesExecutor {
    pool: ThreadPool,
    local_pool: LocalPool,
    local_spawner: LocalSpawner,
    in_flights: Vec<InFlight>,
}

impl TestExecutor for FuturesExecutor {
    fn execute(&mut self, mut test: AsyncTest) {
        let handle = self
            .pool
            .spawn_with_handle(async move { test.run().await })
            .unwrap();
        self.in_flights.push(InFlight {
            handle,
            result: None,
        });
    }

    fn execute_local(&mut self, mut test: LocalAsyncTest) {
        let handle = self
            .local_spawner
            .spawn_local_with_handle(async move { test.run().await })
            .unwrap();
        self.in_flights.push(InFlight {
            handle,
            result: None,
        });
    }

    fn execute_blocking(&mut self, mut test: BlockingTest) {
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
