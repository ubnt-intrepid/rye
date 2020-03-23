//! Abstraction of test execution in the rye.

use crate::{
    reporter::{Outcome, Reporter, TestCaseSummary},
    test::{Location, SectionId, TestCase, TestDesc, TestFn},
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::{cell::Cell, fmt, marker::PhantomData, mem, pin::Pin, ptr::NonNull, sync::Arc};

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

#[derive(Debug)]
enum TerminationReason {
    Skipped { reason: String },
    Failed { location: Location, reason: String },
    AssertionFailed { location: Location, message: String },
}

/// Context values while running the test case.
pub struct Context<'a> {
    desc: &'a TestDesc,
    termination_reason: Option<TerminationReason>,
    target_section: Option<SectionId>,
    current_section: Option<SectionId>,
    #[allow(dead_code)]
    reporter: &'a mut (dyn Reporter + Send),
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<Context<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| tls.set(self.0.take()));
    }
}

impl<'a> Context<'a> {
    fn new(
        desc: &'a TestDesc,
        reporter: &'a mut (dyn Reporter + Send),
        target_section: Option<SectionId>,
    ) -> Self {
        Self {
            desc,
            termination_reason: None,
            target_section,
            current_section: None,
            reporter,
            _marker: PhantomData,
        }
    }

    pub(crate) fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev = TLS_CTX.with(|tls| unsafe {
            let ctx_ptr = mem::transmute::<&mut Self, &mut Context<'static>>(self);
            tls.replace(Some(NonNull::from(ctx_ptr)))
        });
        let _guard = Guard(prev);
        f()
    }

    #[inline]
    pub(crate) async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        #[pin_project]
        struct ScopeAsync<'a, 'ctx, Fut> {
            #[pin]
            fut: Fut,
            ctx: &'a mut Context<'ctx>,
        }

        impl<Fut> Future for ScopeAsync<'_, '_, Fut>
        where
            Fut: Future,
        {
            type Output = Fut::Output;

            fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
                let me = self.project();
                let fut = me.fut;
                me.ctx.scope(|| fut.poll(cx))
            }
        }

        ScopeAsync { fut, ctx: self }.await
    }

    /// Attempt to get a reference to the test context and invoke the provided closure.
    ///
    /// This function returns an `AccessError` if the test context is not available.
    pub fn try_with<F, R>(f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        let ctx_ptr = TLS_CTX.with(|tls| tls.take());
        let _guard = Guard(ctx_ptr);
        let mut ctx_ptr = ctx_ptr.ok_or_else(|| AccessError { _p: () })?;
        Ok(unsafe { f(ctx_ptr.as_mut()) })
    }

    /// Get a reference to the test context and invoke the provided closure.
    ///
    /// # Panics
    /// This function causes a panic if the test context is not available.
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        Self::try_with(f).expect("cannot acquire the test context")
    }

    pub(crate) fn enter_section(&mut self, id: SectionId) -> EnterSection {
        let enabled = self.target_section.map_or(false, |section_id| {
            let section = self
                .desc
                .sections
                .get(&section_id)
                .expect("invalid section id is set");
            section_id == id || section.ancestors.contains(&id)
        });
        let last_section = self.current_section.replace(id);
        EnterSection {
            enabled,
            last_section,
        }
    }

    fn leave_section(&mut self, enter: EnterSection) {
        self.current_section = enter.last_section;
    }

    async fn run_async<Fut>(&mut self, fut: Fut) -> Result<(), Outcome>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let outcome = self.scope_async(fut).await;
        self.check_outcome(outcome)
    }

    fn run_blocking(&mut self, f: fn() -> anyhow::Result<()>) -> Result<(), Outcome> {
        let outcome = self.scope(f);
        self.check_outcome(outcome)
    }

    fn check_outcome(&mut self, result: anyhow::Result<()>) -> Result<(), Outcome> {
        match result {
            Ok(()) => match self.termination_reason.take() {
                Some(TerminationReason::Skipped { reason }) => Err(Outcome::Skipped { reason }),
                Some(TerminationReason::Failed { location, reason }) => {
                    Err(Outcome::Failed { location, reason })
                }
                Some(TerminationReason::AssertionFailed { location, message }) => {
                    Err(Outcome::AssertionFailed { location, message })
                }
                None => Ok(()),
            },
            Err(err) => Err(Outcome::Errored(err)),
        }
    }

    #[inline]
    pub(crate) fn mark_skipped(&mut self, reason: fmt::Arguments<'_>) {
        debug_assert!(self.termination_reason.is_none());
        self.termination_reason.replace(TerminationReason::Skipped {
            reason: reason.to_string(),
        });
    }

    #[inline]
    pub(crate) fn mark_failed(&mut self, location: Location, reason: fmt::Arguments<'_>) {
        debug_assert!(self.termination_reason.is_none());
        self.termination_reason.replace(TerminationReason::Failed {
            location,
            reason: reason.to_string(),
        });
    }

    #[inline]
    pub(crate) fn mark_assertion_failed(
        &mut self,
        location: Location,
        message: fmt::Arguments<'_>,
    ) {
        debug_assert!(self.termination_reason.is_none());
        self.termination_reason
            .replace(TerminationReason::AssertionFailed {
                location,
                message: message.to_string(),
            });
    }
}

#[doc(hidden)]
pub struct EnterSection {
    enabled: bool,
    last_section: Option<SectionId>,
}

impl EnterSection {
    #[doc(hidden)]
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[doc(hidden)]
    #[inline]
    pub fn leave(self) {
        Context::with(|ctx| ctx.leave_section(self))
    }
}

/// The error value returned from `Context::try_with`.
#[derive(Debug)]
pub struct AccessError {
    _p: (),
}

#[cfg(all(test, not(feature = "frameworks")))]
mod tests {
    use super::*;
    use crate::{
        reporter::Summary,
        test::{TestCase, TestDesc, TestFn},
    };
    use futures_executor::block_on;
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(msg: &'static str) {
        let current_section =
            Context::with(|ctx| ctx.current_section.map(|id| ctx.desc.sections[&id].name));
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
