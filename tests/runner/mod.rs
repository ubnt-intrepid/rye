use futures::{
    executor::{LocalPool, LocalSpawner, ThreadPool},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use rye::{
    cli::Session,
    executor::{AsyncTest, BlockingTest, LocalAsyncTest, TestExecutor},
    test::Registration,
};
use std::thread;

pub(crate) fn run_tests(tests: &[&dyn Registration]) {
    rye::cli::install();

    let mut session = Session::from_env(tests);

    let mut local_pool = LocalPool::new();
    let mut executor = FuturesExecutor {
        pool: ThreadPool::new().unwrap(),
        local_spawner: local_pool.spawner(),
    };

    let st = local_pool.run_until(session.run(&mut executor));

    st.exit();
}

struct FuturesExecutor {
    pool: ThreadPool,
    local_spawner: LocalSpawner,
}

impl TestExecutor for FuturesExecutor {
    fn execute(&mut self, mut test: AsyncTest) {
        self.pool
            .spawn(async move {
                test.run().await;
            })
            .unwrap();
    }

    fn execute_local(&mut self, mut test: LocalAsyncTest) {
        self.local_spawner
            .spawn_local(async move {
                test.run().await;
            })
            .unwrap();
    }

    fn execute_blocking(&mut self, mut test: BlockingTest) {
        thread::spawn(move || {
            test.run();
        });
    }
}
