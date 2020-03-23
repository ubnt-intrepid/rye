//! Abstraction of test execution in the rye.

use crate::{
    context::Context,
    reporter::{Outcome, Reporter, TestCaseSummary},
    test::{TestCase, TestDesc, TestFn},
};
use futures_core::future::Future;
use std::sync::Arc;

/// The executor of test cases.
pub trait TestExecutor {
    /// Future for awaiting a result of test execution.
    type Handle: Future<Output = TestCaseSummary>;

    /// Spawn a task to execute the specified test future.
    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static;

    /// Spawn a task to execute the specified test future onto the current thread.
    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static;

    /// Spawn a task to execute the specified test function that may block the running thread.
    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static;
}

impl<T: ?Sized> TestExecutor for &mut T
where
    T: TestExecutor,
{
    type Handle = T::Handle;

    #[inline]
    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static,
    {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static,
    {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static,
    {
        (**self).spawn_blocking(f)
    }
}

impl<T: ?Sized> TestExecutor for Box<T>
where
    T: TestExecutor,
{
    type Handle = T::Handle;

    #[inline]
    fn spawn<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + Send + 'static,
    {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = TestCaseSummary> + 'static,
    {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> TestCaseSummary + Send + 'static,
    {
        (**self).spawn_blocking(f)
    }
}

pub(crate) trait TestExecutorExt: TestExecutor {
    fn spawn_test<R>(&mut self, test: &dyn TestCase, reporter: R) -> Self::Handle
    where
        R: Reporter + Send + 'static,
    {
        let mut inner = TestInner {
            desc: Arc::new(test.desc()),
            reporter: Box::new(reporter),
        };
        match test.test_fn() {
            TestFn::Async(f) => self.spawn(async move { inner.run_async(f).await }),
            TestFn::AsyncLocal(f) => self.spawn_local(async move { inner.run_async(f).await }),
            TestFn::Blocking(f) => self.spawn_blocking(move || inner.run_blocking(f)),
        }
    }
}

impl<E: TestExecutor + ?Sized> TestExecutorExt for E {}

struct TestInner {
    desc: Arc<TestDesc>,
    reporter: Box<dyn Reporter + Send + 'static>,
}

impl TestInner {
    async fn run_async<Fut>(&mut self, f: fn() -> Fut) -> TestCaseSummary
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for section in self.desc.target_sections() {
            if let Err(o) = {
                Context::new(
                    &self.desc, //
                    &mut *self.reporter,
                    section,
                )
                .run_async(f())
                .await
            } {
                outcome = o;
                break;
            }
        }

        self.end_test_case(outcome)
    }

    fn run_blocking(&mut self, f: fn() -> anyhow::Result<()>) -> TestCaseSummary {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for section in self.desc.target_sections() {
            if let Err(o) = {
                Context::new(
                    &self.desc, //
                    &mut *self.reporter,
                    section,
                )
                .run_blocking(f)
            } {
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
            desc: self.desc.clone(),
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
        context::with_tls_context,
        reporter::Summary,
        test::{TestCase, TestDesc, TestFn},
    };
    use futures_executor::block_on;
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(msg: &'static str) {
        let current_section = with_tls_context(|ctx| ctx.current_section_name());
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
        let test_fn = t.test_fn();

        let history = RefCell::new(vec![]);
        let mut state = TestInner {
            desc: Arc::new(desc),
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
        fn test_case() {
            append_history("test");
        }

        let history = run_test(test_case);
        assert_eq!(history, vec![("test", None)]);
    }

    #[test]
    fn one_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case() {
            append_history("setup");

            section!("section1", {
                append_history("section1");
            });

            append_history("teardown");
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
        fn test_case() {
            append_history("setup");

            section!("section1", {
                append_history("section1");
            });

            section!("section2", {
                append_history("section2");
            });

            append_history("teardown");
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
        fn test_case() {
            append_history("setup");

            section!("section1", {
                append_history("section1:setup");

                section!("section2", {
                    append_history("section2");
                });

                section!("section3", {
                    append_history("section3");
                });

                append_history("section1:teardown");
            });

            section!("section4", {
                append_history("section4");
            });

            append_history("teardown");
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
        async fn test_case() {
            use futures_test::future::FutureTestExt as _;

            append_history("setup");
            async {}.pending_once().await;

            section!("section1", {
                append_history("section1:setup");
                async {}.pending_once().await;

                section!("section2", {
                    async {}.pending_once().await;
                    append_history("section2");
                });

                section!("section3", {
                    async {}.pending_once().await;
                    append_history("section3");
                });

                async {}.pending_once().await;
                append_history("section1:teardown");
            });

            section!("section4", {
                async {}.pending_once().await;
                append_history("section4");
            });

            async {}.pending_once().await;
            append_history("teardown");
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
