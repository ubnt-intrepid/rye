use crate::desc::TestDesc;
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{channel::oneshot, executor::ThreadPool, future::Future, task::SpawnExt as _};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use mimicaw::{Outcome, Test, TestRunner};
use std::{fmt::Write as _, panic::AssertUnwindSafe, pin::Pin, sync::Once};

pub struct TestSuite<'a> {
    test_cases: &'a mut Vec<Test<TestData>>,
}

impl TestSuite<'_> {
    #[doc(hidden)] // private API
    pub fn register<F>(&mut self, desc: TestDesc, test_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.test_cases.push(Test::test(
            desc.name,
            TestData {
                desc,
                test_fn: TestFn::Sync(Box::new(test_fn)),
            },
        ));
    }

    #[doc(hidden)] // private API
    pub fn register_async<F, Fut>(&mut self, desc: TestDesc, test_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.test_cases.push(Test::test(
            desc.name,
            TestData {
                desc,
                test_fn: TestFn::Async(Box::new(move || Box::pin(test_fn()))),
            },
        ));
    }
}

pub struct TestData {
    #[allow(dead_code)]
    desc: TestDesc,
    test_fn: TestFn,
}

enum TestFn {
    Sync(Box<dyn Fn() + Send + 'static>),
    Async(
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>,
    ),
}

pub fn run_tests<T>(runner: T, tests: &[&dyn Fn(&mut TestSuite<'_>)])
where
    T: TestRunner<TestData>,
{
    let args = mimicaw::Args::from_env().unwrap_or_else(|st| st.exit());

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        maybe_unwind::set_hook();
    });

    let mut test_cases = vec![];
    for &test in tests {
        test(&mut TestSuite {
            test_cases: &mut test_cases,
        });
    }

    let st = futures::executor::block_on(mimicaw::run_tests(&args, test_cases, runner));
    st.exit();
}

#[derive(Debug)]
pub struct DefaultRunner {
    pool: ThreadPool,
}

impl DefaultRunner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            pool: ThreadPool::new().unwrap(),
        }
    }
}

impl TestRunner<TestData> for DefaultRunner {
    type Future = Pin<Box<dyn Future<Output = Outcome>>>;

    fn run(&mut self, _desc: mimicaw::TestDesc, data: TestData) -> Self::Future {
        let desc = data.desc;
        match data.test_fn {
            TestFn::Sync(f) => {
                let (tx, rx) = oneshot::channel();
                std::thread::spawn(move || {
                    let res = expected(|| maybe_unwind(AssertUnwindSafe(|| desc.run(&f))));
                    let _ = tx.send(res);
                });

                Box::pin(async move {
                    match rx.await {
                        Ok(res) => make_outcome(res),
                        Err(rx_err) => {
                            Outcome::failed().error_message(format!("unknown error: {}", rx_err))
                        }
                    }
                })
            }
            TestFn::Async(f) => {
                let handle = self
                    .pool
                    .spawn_with_handle(async move { desc.run_async(f).await })
                    .unwrap();
                Box::pin(async move {
                    let res = AssertUnwindSafe(handle).maybe_unwind().expected().await;
                    make_outcome(res)
                })
            }
        }
    }
}

fn make_outcome(res: (Result<(), Unwind>, Option<Disappoints>)) -> Outcome {
    match res {
        (Ok(()), None) => Outcome::passed(),
        (Ok(()), Some(disappoints)) => Outcome::failed().error_message(disappoints.to_string()),
        (Err(unwind), disappoints) => {
            let mut msg = String::new();
            let _ = writeln!(&mut msg, "{}", unwind);
            if let Some(disappoints) = disappoints {
                let _ = writeln!(&mut msg, "{}", disappoints);
            }
            Outcome::failed().error_message(msg)
        }
    }
}
