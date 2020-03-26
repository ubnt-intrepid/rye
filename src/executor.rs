//! Abstraction of test execution in the rye.

use crate::{
    context::{Context, ContextPtr},
    report::{Outcome, Reporter, TestCaseSummary},
    test::{TestCase, TestDesc, TestFn, TestPlan},
};
use futures_channel::oneshot;
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use futures_util::future::{FutureExt as _, RemoteHandle};
use pin_project::{pin_project, project};
use std::pin::Pin;

/// The executor of test cases.
pub trait TestExecutor {
    /// Spawn a task to execute the specified test future.
    fn spawn<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    /// Spawn a task to execute the specified test future onto the current thread.
    fn spawn_local<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + 'static;

    /// Spawn a task to execute the specified test function that may block the running thread.
    fn spawn_blocking<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static;
}

impl<T: ?Sized> TestExecutor for &mut T
where
    T: TestExecutor,
{
    #[inline]
    fn spawn<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        (**self).spawn_blocking(f)
    }
}

impl<T: ?Sized> TestExecutor for Box<T>
where
    T: TestExecutor,
{
    #[inline]
    fn spawn<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        (**self).spawn_blocking(f)
    }
}

#[pin_project]
enum HandleKind {
    Async(#[pin] RemoteHandle<TestCaseSummary>),
    Blocking(#[pin] oneshot::Receiver<TestCaseSummary>),
}

impl HandleKind {
    #[project]
    fn poll_inner(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<TestCaseSummary> {
        #[project]
        match self.project() {
            HandleKind::Async(handle) => handle.poll(cx),
            HandleKind::Blocking(rx) => rx.poll(cx).map(|res| res.unwrap()),
        }
    }
}

#[pin_project]
pub(crate) struct Handle {
    #[pin]
    kind: HandleKind,
    desc: &'static TestDesc,
}

impl Future for Handle {
    type Output = TestCaseSummary;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        me.kind.poll_inner(cx)
    }
}

pub(crate) trait TestExecutorExt: TestExecutor {
    fn spawn_test<R>(&mut self, test: &dyn TestCase, reporter: R) -> Handle
    where
        R: Reporter + Send + 'static,
    {
        let mut inner = TestInner {
            desc: test.desc(),
            plans: test.test_plans(),
            reporter: Box::new(reporter),
        };

        let kind = match test.test_fn() {
            TestFn::Async(f) => {
                let (remote, handle) = async move { inner.run_async(f).await }.remote_handle();
                self.spawn(remote);
                HandleKind::Async(handle)
            }
            TestFn::AsyncLocal(f) => {
                let (remote, handle) = async move { inner.run_async(f).await }.remote_handle();
                self.spawn_local(remote);
                HandleKind::Async(handle)
            }
            TestFn::Blocking(f) => {
                let (tx, rx) = oneshot::channel();
                self.spawn_blocking(move || {
                    let summary = inner.run_blocking(f);
                    let _ = tx.send(summary);
                });
                HandleKind::Blocking(rx)
            }
        };

        Handle {
            kind,
            desc: test.desc(),
        }
    }
}

impl<E: TestExecutor + ?Sized> TestExecutorExt for E {}

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
