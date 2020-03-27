//! Abstraction of test execution in the rye.

#![allow(missing_docs)]

use crate::{
    context::{Context, ContextPtr},
    report::{Outcome, Reporter, TestCaseSummary},
    test::{TestCase, TestDesc, TestFn, TestPlan},
};
use futures_channel::oneshot;
use futures_core::{
    future::{BoxFuture, Future, LocalBoxFuture},
    task::{self, Poll},
};
use futures_executor::{LocalPool, LocalSpawner};
use futures_util::task::{LocalSpawnExt as _, SpawnExt as _};
use pin_project::pin_project;
use std::pin::Pin;

/// The executor of test cases.
pub trait TestExecutor {
    /// Spawn a task to execute the specified test future.
    fn spawn(&mut self, testfn: AsyncTestFn);

    /// Spawn a task to execute the specified test future onto the current thread.
    fn spawn_local(&mut self, testfn: LocalAsyncTestFn);

    /// Spawn a task to execute the specified test function that may block the running thread.
    fn spawn_blocking(&mut self, testfn: BlockingTestFn);
}

impl<T: ?Sized> TestExecutor for &mut T
where
    T: TestExecutor,
{
    #[inline]
    fn spawn(&mut self, testfn: AsyncTestFn) {
        (**self).spawn(testfn)
    }

    #[inline]
    fn spawn_local(&mut self, testfn: LocalAsyncTestFn) {
        (**self).spawn_local(testfn)
    }

    #[inline]
    fn spawn_blocking(&mut self, testfn: BlockingTestFn) {
        (**self).spawn_blocking(testfn)
    }
}

impl<T: ?Sized> TestExecutor for Box<T>
where
    T: TestExecutor,
{
    #[inline]
    fn spawn(&mut self, testfn: AsyncTestFn) {
        (**self).spawn(testfn)
    }

    #[inline]
    fn spawn_local(&mut self, testfn: LocalAsyncTestFn) {
        (**self).spawn_local(testfn)
    }

    #[inline]
    fn spawn_blocking(&mut self, testfn: BlockingTestFn) {
        (**self).spawn_blocking(testfn)
    }
}

#[pin_project]
pub(crate) struct Handle {
    #[pin]
    rx: oneshot::Receiver<TestCaseSummary>,
    desc: &'static TestDesc,
}

impl Future for Handle {
    type Output = TestCaseSummary;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        match futures_util::ready!(me.rx.poll(cx)) {
            Ok(summary) => Poll::Ready(summary),
            Err(..) => todo!("report cancellation"),
        }
    }
}

impl dyn TestExecutor + '_ {
    pub(crate) fn spawn_test<R>(&mut self, test: &dyn TestCase, reporter: R) -> Handle
    where
        R: Reporter + Send + 'static,
    {
        let inner = TestInner {
            desc: test.desc(),
            plans: test.test_plans(),
            reporter: Box::new(reporter),
        };

        let (tx, rx) = oneshot::channel();
        match test.test_fn() {
            TestFn::Async(f) => {
                self.spawn(AsyncTestFn { inner, f, tx });
            }
            TestFn::AsyncLocal(f) => {
                self.spawn_local(LocalAsyncTestFn { inner, f, tx });
            }
            TestFn::Blocking(f) => {
                self.spawn_blocking(BlockingTestFn { inner, f, tx });
            }
        }

        Handle {
            rx,
            desc: test.desc(),
        }
    }
}

#[doc(hidden)] // TODO: dox
pub fn block_on<Fut: Future>(f: impl FnOnce(Box<dyn TestExecutor>) -> Fut) -> Fut::Output {
    let mut pool = LocalPool::new();
    let exec = DefaultTestExecutor {
        spawner: pool.spawner(),
    };
    pool.run_until(f(Box::new(exec)))
}

struct DefaultTestExecutor {
    spawner: LocalSpawner,
}

impl TestExecutor for DefaultTestExecutor {
    fn spawn(&mut self, testfn: AsyncTestFn) {
        self.spawner
            .spawn(async move { testfn.run().await })
            .unwrap();
    }

    fn spawn_local(&mut self, testfn: LocalAsyncTestFn) {
        self.spawner
            .spawn_local(async move { testfn.run().await })
            .unwrap();
    }

    fn spawn_blocking(&mut self, testfn: BlockingTestFn) {
        self.spawner.spawn(async move { testfn.run() }).unwrap();
    }
}

pub struct AsyncTestFn {
    inner: TestInner,
    f: fn(ContextPtr) -> BoxFuture<'static, anyhow::Result<()>>,
    tx: oneshot::Sender<TestCaseSummary>,
}

impl AsyncTestFn {
    pub async fn run(mut self) {
        let summary = self.inner.run_async(self.f).await;
        let _ = self.tx.send(summary);
    }
}

pub struct LocalAsyncTestFn {
    inner: TestInner,
    f: fn(ContextPtr) -> LocalBoxFuture<'static, anyhow::Result<()>>,
    tx: oneshot::Sender<TestCaseSummary>,
}

impl LocalAsyncTestFn {
    pub async fn run(mut self) {
        let summary = self.inner.run_async(self.f).await;
        let _ = self.tx.send(summary);
    }
}

pub struct BlockingTestFn {
    inner: TestInner,
    f: fn(ContextPtr) -> anyhow::Result<()>,
    tx: oneshot::Sender<TestCaseSummary>,
}

impl BlockingTestFn {
    pub fn run(mut self) {
        let summary = self.inner.run_blocking(self.f);
        let _ = self.tx.send(summary);
    }
}

struct TestInner {
    desc: &'static TestDesc,
    plans: &'static [TestPlan],
    reporter: Box<dyn Reporter + Send + 'static>,
}

impl TestInner {
    async fn run_async<Fut>(&mut self, f: fn(ContextPtr) -> Fut) -> TestCaseSummary
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for plan in self.plans {
            let mut ctx = Context::new(&mut *self.reporter, plan);
            let result = f(unsafe { ctx.transmute() }).await;
            if let Some(o) = ctx.check_outcome(result) {
                outcome = o;
                break;
            }
        }

        self.end_test_case(outcome)
    }

    fn run_blocking(&mut self, f: fn(ContextPtr) -> anyhow::Result<()>) -> TestCaseSummary {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for plan in self.plans {
            let mut ctx = Context::new(&mut *self.reporter, plan);
            let result = f(unsafe { ctx.transmute() });
            if let Some(o) = ctx.check_outcome(result) {
                outcome = o;
                break;
            }
        }

        self.end_test_case(outcome)
    }

    fn start_test_case(&mut self) {
        self.reporter.test_case_starting(&self.desc);
    }

    fn end_test_case(&mut self, outcome: Outcome) -> TestCaseSummary {
        let summary = TestCaseSummary {
            desc: self.desc,
            outcome,
        };
        self.reporter.test_case_ended(&summary);
        summary
    }
}

#[cfg(all(test, not(feature = "frameworks")))]
mod tests {
    use super::*;
    use crate::{
        report::Summary,
        test::{TestCase, TestDesc, TestFn},
    };
    use futures::executor::block_on;
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(ctx: &mut Context<'_>, msg: &'static str) {
        let current_section = ctx.current_section_name();
        HISTORY.with(|history| history.borrow_mut().push((msg, current_section)));
    }

    struct NullReporter;

    impl Reporter for NullReporter {
        fn test_run_starting(&self, _: &[&dyn TestCase]) {}
        fn test_run_ended(&self, _: &Summary) {}
        fn test_case_starting(&self, _: &TestDesc) {}
        fn test_case_ended(&self, _: &TestCaseSummary) {}
    }

    fn run_test(t: &dyn TestCase) -> Vec<HistoryLog> {
        let desc = t.desc();
        let plans = t.test_plans();
        let test_fn = t.test_fn();

        let history = RefCell::new(vec![]);
        let mut state = TestInner {
            desc,
            plans,
            reporter: Box::new(NullReporter),
        };

        let _ = match test_fn {
            TestFn::Async(f) => block_on(HISTORY.set_async(&history, state.run_async(f))),
            TestFn::AsyncLocal(f) => block_on(HISTORY.set_async(&history, state.run_async(f))),
            TestFn::Blocking(f) => HISTORY.set(&history, || state.run_blocking(f)),
        };

        history.into_inner()
    }

    #[test]
    fn no_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "test");
        }

        let history = run_test(test_case);
        assert_eq!(history, vec![("test", None)]);
    }

    #[test]
    fn one_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1");
            });

            append_history(ctx, "teardown");
        }

        let history = run_test(test_case);
        assert_eq!(
            history,
            vec![
                ("setup", None),
                ("section1", Some("section1")),
                ("teardown", None)
            ]
        );
    }

    #[test]
    fn multi_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1");
            });

            section!(ctx, "section2", {
                append_history(ctx, "section2");
            });

            append_history(ctx, "teardown");
        }

        let history = run_test(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section2", Some("section2")),
                ("teardown", None),
            ]
        );
    }

    #[test]
    fn nested_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1:setup");

                section!(ctx, "section2", {
                    append_history(ctx, "section2");
                });

                section!(ctx, "section3", {
                    append_history(ctx, "section3");
                });

                append_history(ctx, "section1:teardown");
            });

            section!(ctx, "section4", {
                append_history(ctx, "section4");
            });

            append_history(ctx, "teardown");
        }

        let history = run_test(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section2", Some("section2")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section3", Some("section3")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 3
                ("setup", None),
                ("section4", Some("section4")),
                ("teardown", None),
            ]
        );
    }

    #[test]
    fn smoke_async() {
        #[crate::test]
        #[rye(crate = crate)]
        async fn test_case(ctx: &mut Context<'_>) {
            use futures_test::future::FutureTestExt as _;

            append_history(ctx, "setup");
            async {}.pending_once().await;

            section!(ctx, "section1", {
                append_history(ctx, "section1:setup");
                async {}.pending_once().await;

                section!(ctx, "section2", {
                    async {}.pending_once().await;
                    append_history(ctx, "section2");
                });

                section!(ctx, "section3", {
                    async {}.pending_once().await;
                    append_history(ctx, "section3");
                });

                async {}.pending_once().await;
                append_history(ctx, "section1:teardown");
            });

            section!(ctx, "section4", {
                async {}.pending_once().await;
                append_history(ctx, "section4");
            });

            async {}.pending_once().await;
            append_history(ctx, "teardown");
        }

        let history = run_test(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section2", Some("section2")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section3", Some("section3")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 3
                ("setup", None),
                ("section4", Some("section4")),
                ("teardown", None),
            ]
        );
    }
}
