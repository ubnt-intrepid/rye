//! Abstraction of test execution in the rye.

use crate::{
    reporter::{Outcome, Reporter, TestCaseSummary},
    test::{
        imp::{Location, SectionId, TestFn},
        Fallible, Test, TestDesc,
    },
};
use futures::{
    future::{Future, FutureExt as _, LocalFutureObj},
    task::{self, FutureObj, Poll},
};
use pin_project::pin_project;
use std::{
    cell::Cell,
    fmt,
    marker::PhantomData,
    mem,
    panic::{AssertUnwindSafe, PanicInfo},
    pin::Pin,
    ptr::NonNull,
    rc::Rc,
    sync::Arc,
};

/// The executor of test cases.
pub trait TestExecutor {
    /// Future for awaiting a result of test execution.
    type Handle: Future<Output = TestCaseSummary>;

    /// Spawn a task to execute the specified test function.
    fn spawn(&mut self, test: AsyncTest) -> Self::Handle;

    /// Spawn a task to execute the specified test function onto the current thread.
    fn spawn_local(&mut self, test: LocalAsyncTest) -> Self::Handle;

    /// Spawn a taek to execute the specified test function that may block the running thread.
    fn spawn_blocking(&mut self, test: BlockingTest) -> Self::Handle;

    /// Run all test cases and collect their results.
    fn run<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()>;
}

impl<T: ?Sized> TestExecutor for &mut T
where
    T: TestExecutor,
{
    type Handle = T::Handle;

    #[inline]
    fn spawn(&mut self, test: AsyncTest) -> Self::Handle {
        (**self).spawn(test)
    }

    #[inline]
    fn spawn_local(&mut self, test: LocalAsyncTest) -> Self::Handle {
        (**self).spawn_local(test)
    }

    #[inline]
    fn spawn_blocking(&mut self, test: BlockingTest) -> Self::Handle {
        (**self).spawn_blocking(test)
    }

    #[inline]
    fn run<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()>,
    {
        (**self).run(fut)
    }
}

impl<T: ?Sized> TestExecutor for Box<T>
where
    T: TestExecutor,
{
    type Handle = T::Handle;

    #[inline]
    fn spawn(&mut self, test: AsyncTest) -> Self::Handle {
        (**self).spawn(test)
    }

    #[inline]
    fn spawn_local(&mut self, test: LocalAsyncTest) -> Self::Handle {
        (**self).spawn_local(test)
    }

    #[inline]
    fn spawn_blocking(&mut self, test: BlockingTest) -> Self::Handle {
        (**self).spawn_blocking(test)
    }

    #[inline]
    fn run<Fut>(&mut self, fut: Fut)
    where
        Fut: Future<Output = ()>,
    {
        (**self).run(fut)
    }
}

pub(crate) trait TestExecutorExt: TestExecutor {
    fn spawn_test<R>(&mut self, test: &Test, reporter: R) -> Self::Handle
    where
        R: Reporter + Send + 'static,
    {
        let inner = TestInner {
            desc: test.desc.clone(),
            reporter: Box::new(reporter),
        };
        match test.test_fn {
            TestFn::Blocking(f) => self.spawn_blocking(BlockingTest {
                inner,
                f,
                _marker: PhantomData,
            }),
            TestFn::LocalAsync(f) => self.spawn_local(LocalAsyncTest {
                inner,
                f,
                _marker: PhantomData,
            }),
            TestFn::Async(f) => self.spawn(AsyncTest {
                inner,
                f,
                _marker: PhantomData,
            }),
        }
    }
}

impl<E: TestExecutor + ?Sized> TestExecutorExt for E {}

/// Blocking test function.
pub struct BlockingTest {
    inner: TestInner,
    f: fn() -> Box<dyn Fallible>,
    _marker: PhantomData<Cell<()>>,
}

impl BlockingTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &TestDesc {
        &self.inner.desc
    }

    /// Run the test function until all sections are completed.
    pub fn run(&mut self) -> TestCaseSummary {
        self.inner.run_blocking(self.f)
    }
}

/// Asynchronous test function.
pub struct AsyncTest {
    inner: TestInner,
    f: fn() -> FutureObj<'static, Box<dyn Fallible>>,
    _marker: PhantomData<Cell<()>>,
}

impl AsyncTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &TestDesc {
        &self.inner.desc
    }

    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> TestCaseSummary {
        self.inner.run_async(self.f).await
    }
}

/// Asynchronous test function.
///
/// Unlike `AsyncTest`, this function must be executed
/// on the current thread.
pub struct LocalAsyncTest {
    inner: TestInner,
    f: fn() -> LocalFutureObj<'static, Box<dyn Fallible>>,
    _marker: PhantomData<Rc<Cell<()>>>,
}

impl LocalAsyncTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &TestDesc {
        &self.inner.desc
    }

    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> TestCaseSummary {
        self.inner.run_async(self.f).await
    }
}

struct TestInner {
    desc: Arc<TestDesc>,
    reporter: Box<dyn Reporter + Send + 'static>,
}

impl TestInner {
    async fn run_async<Fut>(&mut self, f: fn() -> Fut) -> TestCaseSummary
    where
        Fut: Future<Output = Box<dyn Fallible>>,
    {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for section in self.desc.target_sections() {
            if let Some(o) = {
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

    fn run_blocking(&mut self, f: fn() -> Box<dyn Fallible>) -> TestCaseSummary {
        self.start_test_case();

        let mut outcome = Outcome::Passed;
        for section in self.desc.target_sections() {
            if let Some(o) = {
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
    Panicked { location: Option<Location> },
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

    /// Return whether the test context is available or not.
    #[inline]
    pub fn is_set() -> bool {
        TLS_CTX.with(|tls| tls.get().is_some())
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

    /// Return the name of section currently executing.
    #[inline]
    pub fn section_name(&self) -> Option<&str> {
        self.current_section.map(|id| self.desc.sections[&id].name)
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

    async fn run_async<Fut>(&mut self, fut: Fut) -> Option<Outcome>
    where
        Fut: Future<Output = Box<dyn Fallible>>,
    {
        let outcome = self.scope_async(AssertUnwindSafe(fut).catch_unwind()).await;
        self.check_outcome(outcome)
    }

    fn run_blocking(&mut self, f: fn() -> Box<dyn Fallible>) -> Option<Outcome> {
        let outcome = self.scope(|| std::panic::catch_unwind(f));
        self.check_outcome(outcome)
    }

    fn check_outcome(
        &mut self,
        result: Result<Box<dyn Fallible>, Box<dyn std::any::Any + Send>>,
    ) -> Option<Outcome> {
        match result {
            Ok(fallible) => match fallible.into_result() {
                Ok(()) => None,
                Err(err) => Some(Outcome::Errored(err)),
            },
            Err(panic_payload) => match self.termination_reason.take() {
                Some(TerminationReason::Skipped { reason }) => Some(Outcome::Skipped { reason }),
                Some(TerminationReason::Panicked { location }) => Some(Outcome::Panicked {
                    payload: panic_payload,
                    location: location.expect("the panic location is not available"),
                }),
                None => unreachable!("unexpected termination reason"),
            },
        }
    }

    pub(crate) fn capture_panic_info(&mut self, info: &PanicInfo) {
        self.termination_reason
            .get_or_insert(TerminationReason::Panicked {
                location: info.location().map(|loc| Location::from_std(loc)),
            });
    }

    #[inline]
    pub(crate) fn mark_skipped(&mut self, reason: fmt::Arguments<'_>) {
        debug_assert!(self.termination_reason.is_none());
        self.termination_reason.replace(TerminationReason::Skipped {
            reason: reason.to_string(),
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
#[derive(Debug, thiserror::Error)]
#[error("cannot access the test context outside of the test body")]
pub struct AccessError {
    _p: (),
}

#[cfg(all(test, not(feature = "frameworks")))]
mod tests {
    use super::*;
    use crate::{
        reporter::Summary,
        test::{imp::TestFn, Registry, RegistryError, TestDesc, TestSet},
    };
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(msg: &'static str) {
        let current_section =
            Context::with(|ctx| ctx.current_section.map(|id| ctx.desc.sections[&id].name));
        HISTORY.with(|history| history.borrow_mut().push((msg, current_section)));
    }

    struct MockRegistry<'a>(&'a mut Option<(TestDesc, TestFn)>);
    impl Registry for MockRegistry<'_> {
        fn add_test(&mut self, desc: TestDesc, test_fn: TestFn) -> Result<(), RegistryError> {
            self.0.replace((desc, test_fn));
            Ok(())
        }
    }

    struct NullReporter;

    impl Reporter for NullReporter {
        fn test_run_starting(&self, _: &[Test]) {}
        fn test_run_ended(&self, _: &Summary) {}
        fn test_case_starting(&self, _: &TestDesc) {}
        fn test_case_ended(&self, _: &TestCaseSummary) {}
    }

    fn run_test(r: &dyn TestSet) -> Vec<HistoryLog> {
        let (desc, test_fn) = {
            let mut test = None;
            r.register(&mut MockRegistry(&mut test)).unwrap();
            test.take().expect("test is not registered")
        };

        let history = RefCell::new(vec![]);
        let mut state = TestInner {
            desc: Arc::new(desc),
            reporter: Box::new(NullReporter),
        };

        let _ = match test_fn {
            TestFn::Blocking(f) => HISTORY.set(&history, || state.run_blocking(f)),
            TestFn::Async(f) => {
                futures::executor::block_on(HISTORY.set_async(&history, state.run_async(f)))
            }
            TestFn::LocalAsync(f) => {
                futures::executor::block_on(HISTORY.set_async(&history, state.run_async(f)))
            }
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

        let history = run_test(&test_case::__new());
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

        let history = run_test(&test_case::__new());
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

        let history = run_test(&test_case::__new());
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

        let history = run_test(&test_case::__new());
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

        let history = run_test(&test_case::__new());
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
