//! Abstraction of test execution in the rye.

use crate::{
    reporter::{Reporter, Summary, TestCaseSummary, TestResult},
    test::{
        imp::{SectionId, TestFn, TestFuture},
        Fallible, Test, TestDesc,
    },
};
use futures::{
    future::Future,
    task::{self, Poll},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use pin_project::pin_project;
use std::{
    cell::Cell, marker::PhantomData, mem, panic::AssertUnwindSafe, pin::Pin, ptr::NonNull, rc::Rc,
};

/// The runner of test cases.
pub trait TestRunner {
    /// Spawn a task to execute the specified test function.
    fn spawn(&mut self, test: AsyncTest);

    /// Spawn a task to execute the specified test function onto the current thread.
    fn spawn_local(&mut self, test: LocalAsyncTest);

    /// Spawn a taek to execute the specified test function that may block the running thread.
    fn spawn_blocking(&mut self, test: BlockingTest);

    /// Run all test cases and collect their results.
    fn run(&mut self) -> Summary;
}

impl<T: ?Sized> TestRunner for &mut T
where
    T: TestRunner,
{
    #[inline]
    fn spawn(&mut self, test: AsyncTest) {
        (**self).spawn(test)
    }

    #[inline]
    fn spawn_local(&mut self, test: LocalAsyncTest) {
        (**self).spawn_local(test)
    }

    #[inline]
    fn spawn_blocking(&mut self, test: BlockingTest) {
        (**self).spawn_blocking(test)
    }

    #[inline]
    fn run(&mut self) -> Summary {
        (**self).run()
    }
}

impl<T: ?Sized> TestRunner for Box<T>
where
    T: TestRunner,
{
    #[inline]
    fn spawn(&mut self, test: AsyncTest) {
        (**self).spawn(test)
    }

    #[inline]
    fn spawn_local(&mut self, test: LocalAsyncTest) {
        (**self).spawn_local(test)
    }

    #[inline]
    fn spawn_blocking(&mut self, test: BlockingTest) {
        (**self).spawn_blocking(test)
    }

    #[inline]
    fn run(&mut self) -> Summary {
        (**self).run()
    }
}

pub(crate) trait TestRunnerExt: TestRunner {
    fn spawn_test<R>(&mut self, test: &Test, reporter: R)
    where
        R: Reporter + Send + 'static,
    {
        let inner = TestInner {
            desc: test.desc,
            reporter: Box::new(reporter),
            error_message: None,
        };
        match test.test_fn {
            TestFn::Blocking { f } => self.spawn_blocking(BlockingTest {
                inner,
                f,
                _marker: PhantomData,
            }),
            TestFn::Async { f, local } => {
                if local {
                    self.spawn_local(LocalAsyncTest {
                        inner,
                        f,
                        _marker: PhantomData,
                    })
                } else {
                    self.spawn(AsyncTest {
                        inner,
                        f,
                        _marker: PhantomData,
                    })
                }
            }
        }
    }
}

impl<E: TestRunner + ?Sized> TestRunnerExt for E {}

/// Blocking test function.
pub struct BlockingTest {
    inner: TestInner,
    f: fn() -> Box<dyn Fallible>,
    _marker: PhantomData<Cell<()>>,
}

impl BlockingTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.inner.desc
    }

    /// Run the test function until all sections are completed.
    pub fn run(&mut self) -> TestCaseSummary {
        self.inner.run_blocking(self.f)
    }
}

/// Asynchronous test function.
pub struct AsyncTest {
    inner: TestInner,
    f: fn() -> TestFuture,
    _marker: PhantomData<Cell<()>>,
}

impl AsyncTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.inner.desc
    }

    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> TestCaseSummary {
        self.inner
            .run_async(self.f, TestFuture::into_future_obj)
            .await
    }
}

/// Asynchronous test function.
///
/// Unlike `AsyncTest`, this function must be executed
/// on the current thread.
pub struct LocalAsyncTest {
    inner: TestInner,
    f: fn() -> TestFuture,
    _marker: PhantomData<Rc<Cell<()>>>,
}

impl LocalAsyncTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.inner.desc
    }

    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> TestCaseSummary {
        self.inner.run_async(self.f, std::convert::identity).await
    }
}

struct TestInner {
    desc: &'static TestDesc,
    reporter: Box<dyn Reporter + Send + 'static>,
    error_message: Option<String>,
}

impl TestInner {
    async fn run_async<F, Fut>(&mut self, f: fn() -> TestFuture, conv: F) -> TestCaseSummary
    where
        F: Fn(TestFuture) -> Fut,
        Fut: Future<Output = Box<dyn Fallible>>,
    {
        self.start();

        if self.desc.leaf_sections.is_empty() {
            self.run_section_async(None, f, &conv).await;
        } else {
            for &section in self.desc.leaf_sections {
                self.run_section_async(Some(section), f, &conv).await;
            }
        }

        self.end()
    }

    fn run_blocking(&mut self, f: fn() -> Box<dyn Fallible>) -> TestCaseSummary {
        self.start();

        if self.desc.leaf_sections.is_empty() {
            self.run_section_blocking(None, f);
        } else {
            for &section in self.desc.leaf_sections {
                self.run_section_blocking(Some(section), f);
            }
        }

        self.end()
    }

    async fn run_section_async<F, Fut>(
        &mut self,
        section: Option<SectionId>,
        f: fn() -> TestFuture,
        conv: F,
    ) where
        F: FnOnce(TestFuture) -> Fut,
        Fut: Future<Output = Box<dyn Fallible>>,
    {
        let fut = conv(f());
        let result = AssertUnwindSafe(self.context(section).scope_async(fut))
            .maybe_unwind()
            .await;
        self.collect_result(result);
    }

    fn run_section_blocking(&mut self, section: Option<SectionId>, f: fn() -> Box<dyn Fallible>) {
        let mut ctx = self.context(section);
        let result = maybe_unwind(AssertUnwindSafe(|| ctx.scope(f)));
        self.collect_result(result);
    }

    fn context(&mut self, target_section: Option<SectionId>) -> Context<'_> {
        Context {
            desc: &self.desc,
            target_section,
            current_section: None,
            event_handler: &mut self.reporter,
            _marker: PhantomData,
        }
    }

    fn start(&mut self) {
        self.reporter.test_case_starting(self.desc);
    }

    fn end(&mut self) -> TestCaseSummary {
        let error_message = self.error_message.take();
        let summary = TestCaseSummary {
            desc: self.desc,
            result: if error_message.is_none() {
                TestResult::Passed
            } else {
                TestResult::Failed
            },
            error_message,
        };
        self.reporter.test_case_ended(&summary);
        summary
    }

    fn collect_result(&mut self, result: Result<Box<dyn Fallible>, Unwind>) {
        match result {
            Ok(result) => {
                if let Some(msg) = result.error_message() {
                    *self.error_message.get_or_insert_with(Default::default) +=
                        &format!("{:?}", msg);
                }
            }
            Err(unwind) => {
                *self.error_message.get_or_insert_with(Default::default) += &unwind.to_string();
            }
        }
    }
}

/// Context values while running the test case.
pub struct Context<'a> {
    desc: &'a TestDesc,
    target_section: Option<SectionId>,
    current_section: Option<SectionId>,
    #[allow(dead_code)]
    event_handler: &'a mut (dyn Reporter + Send),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{imp::TestFn, Registry, RegistryError, Test, TestSet};
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(msg: &'static str) {
        let current_section =
            Context::with(|ctx| ctx.current_section.map(|id| ctx.desc.sections[&id].name));
        HISTORY.with(|history| history.borrow_mut().push((msg, current_section)));
    }

    struct MockRegistry<'a>(&'a mut Option<Test>);
    impl Registry for MockRegistry<'_> {
        fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
            self.0.replace(test);
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
        let test = {
            let mut test = None;
            r.register(&mut MockRegistry(&mut test)).unwrap();
            test.take().expect("test is not registered")
        };

        let history = RefCell::new(vec![]);
        let mut state = TestInner {
            desc: test.desc,
            reporter: Box::new(NullReporter),
            error_message: None,
        };

        let _ = match test.test_fn {
            TestFn::Blocking { f } => HISTORY.set(&history, || state.run_blocking(f)),
            TestFn::Async { f, .. } => futures::executor::block_on(
                HISTORY.set_async(&history, state.run_async(f, std::convert::identity)),
            ),
        };

        history.into_inner()
    }

    mod no_section {
        use super::*;

        #[crate::test]
        #[rye(crate = "crate")]
        fn test_case() {
            append_history("test");
        }

        #[test]
        fn test() {
            let history = run_test(&test_case::__TESTS);
            assert_eq!(history, vec![("test", None)]);
        }
    }

    mod one_section {
        use super::*;

        #[crate::test]
        #[rye(crate = "crate")]
        fn test_case() {
            append_history("setup");

            section!("section1", {
                append_history("section1");
            });

            append_history("teardown");
        }

        #[test]
        fn test() {
            let history = run_test(&test_case::__TESTS);
            assert_eq!(
                history,
                vec![
                    ("setup", None),
                    ("section1", Some("section1")),
                    ("teardown", None)
                ]
            );
        }
    }

    mod multi_section {
        use super::*;

        #[crate::test]
        #[rye(crate = "crate")]
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

        #[test]
        fn test() {
            let history = run_test(&test_case::__TESTS);
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
    }

    mod nested_section {
        use super::*;

        #[crate::test]
        #[rye(crate = "crate")]
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

            append_history("test");

            section!("section4", {
                append_history("section4");
            });

            append_history("teardown");
        }

        #[test]
        fn test() {
            let history = run_test(&test_case::__TESTS);
            assert_eq!(
                history,
                vec![
                    // phase 1
                    ("setup", None),
                    ("section1:setup", Some("section1")),
                    ("section2", Some("section2")),
                    ("section1:teardown", Some("section1")),
                    ("test", None),
                    ("teardown", None),
                    // phase 2
                    ("setup", None),
                    ("section1:setup", Some("section1")),
                    ("section3", Some("section3")),
                    ("section1:teardown", Some("section1")),
                    ("test", None),
                    ("teardown", None),
                    // phase 3
                    ("setup", None),
                    ("test", None),
                    ("section4", Some("section4")),
                    ("teardown", None),
                ]
            );
        }
    }

    mod smoke_async {
        use super::*;

        #[crate::test]
        #[rye(crate = "crate")]
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

            append_history("test");
            async {}.pending_once().await;

            section!("section4", {
                async {}.pending_once().await;
                append_history("section4");
            });

            async {}.pending_once().await;
            append_history("teardown");
        }

        #[test]
        fn test() {
            let history = run_test(&test_case::__TESTS);
            assert_eq!(
                history,
                vec![
                    // phase 1
                    ("setup", None),
                    ("section1:setup", Some("section1")),
                    ("section2", Some("section2")),
                    ("section1:teardown", Some("section1")),
                    ("test", None),
                    ("teardown", None),
                    // phase 2
                    ("setup", None),
                    ("section1:setup", Some("section1")),
                    ("section3", Some("section3")),
                    ("section1:teardown", Some("section1")),
                    ("test", None),
                    ("teardown", None),
                    // phase 3
                    ("setup", None),
                    ("test", None),
                    ("section4", Some("section4")),
                    ("teardown", None),
                ]
            );
        }
    }
}
