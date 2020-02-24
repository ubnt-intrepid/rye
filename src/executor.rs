//! Abstraction of test execution in the rye.

use crate::{
    context::TestContext,
    test::{Test, TestDesc, TestFn, TestFuture},
};
use futures::future::Future;
use std::{cell::Cell, marker::PhantomData};

/// Test executor.
pub trait TestExecutor {
    /// The type of handle for awaiting the test completion.
    type Handle;

    /// Execute a test body.
    fn execute(&mut self, test: TestBody) -> Self::Handle;

    /// Execute an asynchronous test body.
    fn execute_async(&mut self, test: AsyncTestBody) -> Self::Handle;
}

impl Test {
    /// Execute the test function using the specified test executor.
    pub fn execute<E: ?Sized>(&self, exec: &mut E) -> E::Handle
    where
        E: TestExecutor,
    {
        let desc = self.desc.clone();
        match self.test_fn {
            TestFn::SyncTest(f) => exec.execute(TestBody {
                desc,
                f,
                _marker: PhantomData,
            }),
            TestFn::AsyncTest { f, local } => exec.execute_async(AsyncTestBody {
                desc,
                f,
                local,
                _marker: PhantomData,
            }),
        }
    }
}

pub struct TestBody {
    desc: TestDesc,
    f: fn(),
    _marker: PhantomData<Cell<()>>,
}

impl TestBody {
    pub fn run(&mut self) {
        if self.desc.leaf_sections.is_empty() {
            TestContext {
                desc: &self.desc,
                target_section: None,
                current_section: None,
            }
            .scope(&self.f);
        } else {
            for &section in &self.desc.leaf_sections {
                TestContext {
                    desc: &self.desc,
                    target_section: Some(section),
                    current_section: None,
                }
                .scope(&self.f);
            }
        }
    }
}

pub struct AsyncTestBody {
    desc: TestDesc,
    f: fn() -> TestFuture,
    local: bool,
    _marker: PhantomData<Cell<()>>,
}

impl AsyncTestBody {
    async fn run_inner<F, Fut>(&mut self, f: F)
    where
        F: Fn(TestFuture) -> Fut,
        Fut: Future<Output = ()>,
    {
        if self.desc.leaf_sections.is_empty() {
            let fut = f((self.f)());
            TestContext {
                desc: &self.desc,
                target_section: None,
                current_section: None,
            }
            .scope_async(fut)
            .await;
        } else {
            for &section in &self.desc.leaf_sections {
                let fut = f((self.f)());
                TestContext {
                    desc: &self.desc,
                    target_section: Some(section),
                    current_section: None,
                }
                .scope_async(fut)
                .await;
            }
        }
    }

    #[inline]
    pub fn run(&mut self) -> impl Future<Output = ()> + Send + '_ {
        self.run_inner(TestFuture::into_future_obj)
    }

    #[inline]
    pub fn run_local(&mut self) -> impl Future<Output = ()> + '_ {
        self.run_inner(std::convert::identity)
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        self.local
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
            TestFn::SyncTest(f) => HISTORY.set(&history, || {
                TestBody {
                    desc: test.desc,
                    f,
                    _marker: PhantomData,
                }
                .run()
            }),
            TestFn::AsyncTest { f, local } => {
                futures::executor::block_on(HISTORY.set_async(&history, async {
                    AsyncTestBody {
                        desc: test.desc,
                        f,
                        local,
                        _marker: PhantomData,
                    }
                    .run()
                    .await
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
