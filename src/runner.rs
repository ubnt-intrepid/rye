//! Abstraction of test execution in the rye.

pub(crate) mod context;
pub(crate) mod result;

pub use context::{AccessError, Context};
pub use result::{Summary, TestCaseResult};

use crate::{
    reporter::Reporter,
    test::{
        imp::{TestFn, TestFuture},
        Fallible, Test, TestDesc,
    },
};
use futures::future::Future;
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _};
use std::{cell::Cell, marker::PhantomData, panic::AssertUnwindSafe};

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
        let desc = test.desc;
        let reporter = Box::new(reporter);
        match test.test_fn {
            TestFn::Blocking { f } => self.spawn_blocking(BlockingTest {
                desc,
                reporter,
                f,
                _marker: PhantomData,
            }),
            TestFn::Async { f, local } => {
                let inner = AsyncTestInner { desc, reporter, f };
                if local {
                    self.spawn_local(LocalAsyncTest {
                        inner,
                        _marker: PhantomData,
                    })
                } else {
                    self.spawn(AsyncTest {
                        inner,
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
    desc: &'static TestDesc,
    f: fn() -> Box<dyn Fallible>,
    reporter: Box<dyn Reporter + Send + 'static>,
    _marker: PhantomData<Cell<()>>,
}

impl BlockingTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.desc
    }

    /// Run the test function until all sections are completed.
    pub fn run(&mut self) -> TestCaseResult {
        self.reporter.test_case_starting(self.desc);

        let mut error_message = None::<String>;

        if self.desc.leaf_sections.is_empty() {
            let result = maybe_unwind(AssertUnwindSafe(|| {
                Context {
                    desc: &self.desc,
                    target_section: None,
                    current_section: None,
                    event_handler: &mut self.reporter,
                    _marker: PhantomData,
                }
                .scope(&self.f)
            }));
            match result {
                Ok(result) => {
                    if let Some(msg) = result.error_message() {
                        *error_message.get_or_insert_with(Default::default) +=
                            &format!("{:?}", msg);
                    }
                }
                Err(unwind) => {
                    *error_message.get_or_insert_with(Default::default) += &unwind.to_string();
                }
            }
        } else {
            for &section in &self.desc.leaf_sections {
                let result = maybe_unwind(AssertUnwindSafe(|| {
                    Context {
                        desc: &self.desc,
                        target_section: Some(section),
                        current_section: None,
                        event_handler: &mut self.reporter,
                        _marker: PhantomData,
                    }
                    .scope(&self.f)
                }));
                match result {
                    Ok(result) => {
                        if let Some(msg) = result.error_message() {
                            *error_message.get_or_insert_with(Default::default) +=
                                &format!("{:?}", msg);
                        }
                    }
                    Err(unwind) => {
                        *error_message.get_or_insert_with(Default::default) += &unwind.to_string();
                    }
                }
            }
        }

        let result = TestCaseResult {
            desc: self.desc,
            result: if error_message.is_none() {
                result::TestResult::Passed
            } else {
                result::TestResult::Failed
            },
            error_message,
        };
        self.reporter.test_case_ended(&result);

        result
    }
}

struct AsyncTestInner {
    desc: &'static TestDesc,
    f: fn() -> TestFuture,
    reporter: Box<dyn Reporter + Send + 'static>,
}

impl AsyncTestInner {
    async fn run<F, Fut>(&mut self, f: F) -> TestCaseResult
    where
        F: Fn(TestFuture) -> Fut,
        Fut: Future<Output = Box<dyn Fallible>>,
    {
        trait FutureAssertUnwindSafeExt: Future + Sized {
            fn assert_unwind_safe(self) -> AssertUnwindSafe<Self> {
                AssertUnwindSafe(self)
            }
        }

        impl<F: Future> FutureAssertUnwindSafeExt for F {}

        self.reporter.test_case_starting(self.desc);

        let mut error_message = None::<String>;

        if self.desc.leaf_sections.is_empty() {
            let fut = f((self.f)());
            let result = Context {
                desc: &self.desc,
                target_section: None,
                current_section: None,
                event_handler: &mut self.reporter,
                _marker: PhantomData,
            }
            .scope_async(fut)
            .assert_unwind_safe()
            .maybe_unwind()
            .await;
            match result {
                Ok(result) => {
                    if let Some(msg) = result.error_message() {
                        *error_message.get_or_insert_with(Default::default) +=
                            &format!("{:?}", msg);
                    }
                }
                Err(unwind) => {
                    *error_message.get_or_insert_with(Default::default) += &unwind.to_string();
                }
            }
        } else {
            for &section in &self.desc.leaf_sections {
                let fut = f((self.f)());
                let result = Context {
                    desc: &self.desc,
                    target_section: Some(section),
                    current_section: None,
                    event_handler: &mut self.reporter,
                    _marker: PhantomData,
                }
                .scope_async(fut)
                .assert_unwind_safe()
                .maybe_unwind()
                .await;

                match result {
                    Ok(result) => {
                        if let Some(msg) = result.error_message() {
                            *error_message.get_or_insert_with(Default::default) +=
                                &format!("{:?}", msg);
                        }
                    }
                    Err(unwind) => {
                        *error_message.get_or_insert_with(Default::default) += &unwind.to_string();
                    }
                }
            }
        }

        let result = TestCaseResult {
            desc: self.desc,
            result: if error_message.is_none() {
                result::TestResult::Passed
            } else {
                result::TestResult::Failed
            },
            error_message,
        };
        self.reporter.test_case_ended(&result);

        result
    }
}

/// Asynchronous test function.
pub struct AsyncTest {
    inner: AsyncTestInner,
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
    pub async fn run(&mut self) -> TestCaseResult {
        self.inner.run(TestFuture::into_future_obj).await
    }
}

/// Asynchronous test function.
///
/// Unlike `AsyncTest`, this function must be executed
/// on the current thread.
pub struct LocalAsyncTest {
    inner: AsyncTestInner,
    _marker: PhantomData<std::rc::Rc<std::cell::Cell<()>>>,
}

impl LocalAsyncTest {
    #[allow(missing_docs)]
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.inner.desc
    }

    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> TestCaseResult {
        self.inner.run(std::convert::identity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{imp::TestFn, Registration, Registry, RegistryError, Test};
    use futures::task::{self, Poll};
    use scoped_tls::{scoped_thread_local, ScopedKey};
    use std::{cell::RefCell, marker::PhantomData, pin::Pin};

    trait ScopedKeyExt<T> {
        fn set_async<'a, Fut>(&'static self, t: &'a T, fut: Fut) -> SetAsync<'a, T, Fut>
        where
            T: 'static,
            Fut: Future;
    }

    impl<T> ScopedKeyExt<T> for ScopedKey<T> {
        fn set_async<'a, Fut>(&'static self, t: &'a T, fut: Fut) -> SetAsync<'a, T, Fut>
        where
            T: 'static,
            Fut: Future,
        {
            SetAsync { key: self, t, fut }
        }
    }

    #[pin_project::pin_project]
    struct SetAsync<'a, T: 'static, Fut> {
        key: &'static ScopedKey<T>,
        t: &'a T,
        #[pin]
        fut: Fut,
    }

    impl<T, Fut> Future for SetAsync<'_, T, Fut>
    where
        Fut: Future,
    {
        type Output = Fut::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
            let me = self.project();
            let key = me.key;
            let t = *me.t;
            let fut = me.fut;
            key.set(t, || fut.poll(cx))
        }
    }

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
        fn test_case_ended(&self, _: &TestCaseResult) {}
    }

    fn run_test(r: &dyn Registration) -> Vec<HistoryLog> {
        let test = {
            let mut test = None;
            r.register(&mut MockRegistry(&mut test)).unwrap();
            test.take().expect("test is not registered")
        };

        let history = RefCell::new(vec![]);
        match test.test_fn {
            TestFn::Blocking { f } => HISTORY.set(&history, || {
                BlockingTest {
                    desc: test.desc,
                    reporter: Box::new(NullReporter),
                    f,
                    _marker: PhantomData,
                }
                .run();
            }),
            TestFn::Async { f, .. } => {
                futures::executor::block_on(HISTORY.set_async(&history, async {
                    AsyncTestInner {
                        desc: test.desc,
                        reporter: Box::new(NullReporter),
                        f,
                    }
                    .run(std::convert::identity)
                    .await;
                }))
            }
        }

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
            let history = run_test(&test_case::__REGISTRATION);
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
            let history = run_test(&test_case::__REGISTRATION);
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
            let history = run_test(&test_case::__REGISTRATION);
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
            let history = run_test(&test_case::__REGISTRATION);
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
            let history = run_test(&test_case::__REGISTRATION);
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
