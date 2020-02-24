//! Abstraction of test execution in the rye.

use crate::{
    context::TestContext,
    test::{Test, TestDesc, TestFn, TestFuture, TestResult},
};
use futures::future::Future;
use std::{cell::Cell, marker::PhantomData};

/// Test executor.
pub trait TestExecutor {
    /// The type of handle for awaiting the test completion.
    type Handle;

    /// Execute an asynchronous test function.
    fn execute(&mut self, test: AsyncTest) -> Self::Handle;

    /// Execute an asynchronous test function on the current thread.
    fn execute_local(&mut self, test: LocalAsyncTest) -> Self::Handle;

    /// Execute a blocking test function.
    fn execute_blocking(&mut self, test: BlockingTest) -> Self::Handle;
}

impl Test {
    /// Execute the test function using the specified test executor.
    pub fn execute<E: ?Sized>(&self, exec: &mut E) -> E::Handle
    where
        E: TestExecutor,
    {
        let desc = self.desc.clone();
        match self.test_fn {
            TestFn::Blocking { f } => exec.execute_blocking(BlockingTest {
                desc,
                f,
                _marker: PhantomData,
            }),
            TestFn::Async { f, local } => {
                let inner = AsyncTestInner { desc, f };
                if local {
                    exec.execute_local(LocalAsyncTest {
                        inner,
                        _marker: PhantomData,
                    })
                } else {
                    exec.execute(AsyncTest {
                        inner,
                        _marker: PhantomData,
                    })
                }
            }
        }
    }
}

/// Blocking test function.
pub struct BlockingTest {
    desc: TestDesc,
    f: fn() -> Box<dyn TestResult>,
    _marker: PhantomData<Cell<()>>,
}

impl BlockingTest {
    /// Run the test function until all sections are completed.
    pub fn run(&mut self) -> Box<dyn TestResult> {
        if self.desc.leaf_sections.is_empty() {
            TestContext {
                desc: &self.desc,
                target_section: None,
                current_section: None,
            }
            .scope(&self.f)
        } else {
            for &section in &self.desc.leaf_sections {
                let term = TestContext {
                    desc: &self.desc,
                    target_section: Some(section),
                    current_section: None,
                }
                .scope(&self.f);
                if !term.is_success() {
                    return term;
                }
            }
            Box::new(())
        }
    }
}

struct AsyncTestInner {
    desc: TestDesc,
    f: fn() -> TestFuture,
}

impl AsyncTestInner {
    async fn run<F, Fut>(&mut self, f: F) -> Box<dyn TestResult>
    where
        F: Fn(TestFuture) -> Fut,
        Fut: Future<Output = Box<dyn TestResult>>,
    {
        if self.desc.leaf_sections.is_empty() {
            let fut = f((self.f)());
            TestContext {
                desc: &self.desc,
                target_section: None,
                current_section: None,
            }
            .scope_async(fut)
            .await
        } else {
            for &section in &self.desc.leaf_sections {
                let fut = f((self.f)());
                let term = TestContext {
                    desc: &self.desc,
                    target_section: Some(section),
                    current_section: None,
                }
                .scope_async(fut)
                .await;
                if !term.is_success() {
                    return term;
                }
            }
            Box::new(())
        }
    }
}

/// Asynchronous test function.
pub struct AsyncTest {
    inner: AsyncTestInner,
    _marker: PhantomData<Cell<()>>,
}

impl AsyncTest {
    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> Box<dyn TestResult> {
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
    /// Run the test function until all sections are completed.
    #[inline]
    pub async fn run(&mut self) -> Box<dyn TestResult> {
        self.inner.run(std::convert::identity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        registration::{Registration, Registry, RegistryError},
        test::{Test, TestFn},
    };
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
            TestContext::with(|ctx| ctx.current_section.map(|id| ctx.desc.sections[&id].name));
        HISTORY.with(|history| history.borrow_mut().push((msg, current_section)));
    }

    struct MockRegistry<'a>(&'a mut Option<Test>);
    impl Registry for MockRegistry<'_> {
        fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
            self.0.replace(test);
            Ok(())
        }
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
                    f,
                    _marker: PhantomData,
                }
                .run();
            }),
            TestFn::Async { f, .. } => {
                futures::executor::block_on(HISTORY.set_async(&history, async {
                    AsyncTestInner { desc: test.desc, f }
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
        #[rye(rye_path = "crate")]
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
        #[rye(rye_path = "crate")]
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
        #[rye(rye_path = "crate")]
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
        #[rye(rye_path = "crate")]
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
        #[rye(rye_path = "crate")]
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
