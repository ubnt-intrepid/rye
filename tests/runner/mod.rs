use futures::{
    executor::{LocalPool, LocalSpawner, ThreadPool},
    future::{BoxFuture, Future},
    task::{LocalSpawnExt as _, SpawnExt as _},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use rye::{
    executor::{AsyncTest, BlockingTest, LocalAsyncTest, TestExecutor},
    test::{Registration, TestResult},
};
use std::{
    error, fmt,
    panic::{self, AssertUnwindSafe, PanicInfo},
    sync::Once,
    thread,
};

fn panic_hook(info: &PanicInfo) {
    maybe_unwind::capture_panic_info(info);
}

pub(crate) fn run_tests(tests: &[&dyn Registration]) {
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(panic_hook));
    });

    rye::cli::run_tests(tests, |session| {
        let mut local_pool = LocalPool::new();
        let mut executor = FuturesExecutor {
            pool: ThreadPool::new().unwrap(),
            local_spawner: local_pool.spawner(),
        };
        local_pool.run_until(session.execute_tests(&mut executor));
    });
}

struct FuturesExecutor {
    pool: ThreadPool,
    local_spawner: LocalSpawner,
}

impl TestExecutor for FuturesExecutor {
    type Handle = BoxFuture<'static, Result<(), ErrorMessage>>;

    fn execute(&mut self, mut test: AsyncTest) -> Self::Handle {
        let handle = self
            .pool
            .spawn_with_handle(async move { run_test(test.run()).await });
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => Err(ErrorMessage(
                    format!("internal error: {}", spawn_err).into(),
                )),
            }
        })
    }

    fn execute_local(&mut self, mut test: LocalAsyncTest) -> Self::Handle {
        let handle = self
            .local_spawner
            .spawn_local_with_handle(async move { run_test(test.run()).await });
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => Err(ErrorMessage(format!("unknown error: {}", spawn_err).into())),
            }
        })
    }

    fn execute_blocking(&mut self, mut test: BlockingTest) -> Self::Handle {
        let (tx, rx) = futures::channel::oneshot::channel();
        thread::spawn(move || {
            let res = maybe_unwind(AssertUnwindSafe(|| test.run()));
            let _ = tx.send(make_outcome(res));
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|rx_err| {
                Err(ErrorMessage(format!("internal error: {}", rx_err).into()))
            })
        })
    }
}

async fn run_test<Fut>(fut: Fut) -> Result<(), ErrorMessage>
where
    Fut: Future<Output = Box<dyn TestResult>>,
{
    let res = AssertUnwindSafe(fut).maybe_unwind().await;
    make_outcome(res)
}

fn make_outcome(res: Result<Box<dyn TestResult>, Unwind>) -> Result<(), ErrorMessage> {
    let error_message = match res {
        Ok(term) if term.is_success() => None,
        Ok(term) => Some(
            term.error_message()
                .map_or("<unknown>".into(), |msg| format!("{:?}", msg)),
        ),
        Err(unwind) => Some(unwind.to_string()),
    };

    match error_message {
        None => Ok(()),
        Some(msg) => Err(ErrorMessage(msg.into())),
    }
}

struct ErrorMessage(Box<dyn error::Error + Send + Sync>);

impl fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt::Debug::fmt(&*self.0, f)
        } else {
            fmt::Display::fmt(&*self.0, f)
        }
    }
}
