use crate::test::{SectionId, Test, TestDesc, TestFn};
use futures::{
    future::{BoxFuture, Future},
    task::{self, Poll},
};
use pin_project::pin_project;
use std::{cell::Cell, mem, pin::Pin, ptr::NonNull};

pub trait TestExecutor {
    type Handle;

    fn execute<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn execute_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() + Send + 'static;
}

impl Test {
    /// Execute the test function using the specified test executor.
    pub fn execute<E: ?Sized>(&self, exec: &mut E) -> E::Handle
    where
        E: TestExecutor,
    {
        let desc = self.desc.clone();
        match self.test_fn {
            TestFn::SyncTest(f) => exec.execute_blocking(move || run_sync(&desc, f)),
            TestFn::AsyncTest(f) => exec.execute(async move {
                run_async(&desc, f).await;
            }),
        }
    }
}

fn run_sync(desc: &TestDesc, f: fn()) {
    if desc.leaf_sections.is_empty() {
        TestContext {
            desc: &desc,
            section: None,
        }
        .scope(&f);
    } else {
        for &section in &desc.leaf_sections {
            TestContext {
                desc: &desc,
                section: Some(section),
            }
            .scope(&f);
        }
    }
}

async fn run_async(desc: &TestDesc, f: fn() -> BoxFuture<'static, ()>) {
    if desc.leaf_sections.is_empty() {
        TestContext {
            desc: &desc,
            section: None,
        }
        .scope_async(f())
        .await;
    } else {
        for &section in &desc.leaf_sections {
            TestContext {
                desc: &desc,
                section: Some(section),
            }
            .scope_async(f())
            .await;
        }
    }
}

pub(crate) struct TestContext<'a> {
    desc: &'a TestDesc,
    section: Option<SectionId>,
}

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<TestContext<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<TestContext<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| tls.set(self.0.take()));
    }
}

impl<'a> TestContext<'a> {
    fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev = TLS_CTX.with(|tls| unsafe {
            let ctx_ptr = mem::transmute::<&mut Self, &mut TestContext<'static>>(self);
            tls.replace(Some(NonNull::from(ctx_ptr)))
        });
        let _guard = Guard(prev);
        f()
    }

    #[inline]
    async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        #[pin_project]
        struct ScopeAsync<'a, 'ctx, Fut> {
            #[pin]
            fut: Fut,
            ctx: &'a mut TestContext<'ctx>,
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

    fn try_with<F, R>(f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&mut TestContext<'_>) -> R,
    {
        let ctx_ptr = TLS_CTX.with(|tls| tls.take());
        let _guard = Guard(ctx_ptr);
        let mut ctx_ptr = ctx_ptr.ok_or_else(|| AccessError { _p: () })?;
        Ok(unsafe { f(ctx_ptr.as_mut()) })
    }

    pub(crate) fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut TestContext<'_>) -> R,
    {
        Self::try_with(f).expect("cannot acquire the test context")
    }

    pub(crate) fn is_target_section(&self, id: SectionId) -> bool {
        self.section.map_or(false, |section_id| {
            let section = self
                .desc
                .sections
                .get(&section_id)
                .expect("invalid section id is set");
            section_id == id || section.ancestors.contains(&id)
        })
    }
}

#[derive(Debug)]
pub(crate) struct AccessError {
    _p: (),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registration::{Registration, Registry, RegistryError};
    use scoped_tls::{scoped_thread_local, ScopedKey};
    use std::cell::RefCell;

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

    scoped_thread_local!(static HISTORY: RefCell<Vec<&'static str>>);

    fn append_history(v: &'static str) {
        HISTORY.with(|history| history.borrow_mut().push(v));
    }

    struct MockRegistry<'a>(&'a mut Option<Test>);
    impl Registry for MockRegistry<'_> {
        fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
            self.0.replace(test);
            Ok(())
        }
    }

    fn run_test(r: &dyn Registration) -> Vec<&'static str> {
        let test = {
            let mut test = None;
            r.register(&mut MockRegistry(&mut test)).unwrap();
            test.take().expect("test is not registered")
        };

        let history = RefCell::new(vec![]);
        match test.test_fn {
            TestFn::SyncTest(f) => HISTORY.set(&history, || run_sync(&test.desc, f)),
            TestFn::AsyncTest(f) => futures::executor::block_on(
                HISTORY.set_async(&history, async { run_async(&test.desc, f).await }),
            ),
        }

        history.into_inner()
    }

    mod no_section {
        use super::*;

        #[crate::test(rye_path = "crate")]
        fn test_case() {
            append_history("test");
        }

        #[test]
        fn test() {
            let history = run_test(&test_case::__REGISTRATION);
            assert_eq!(history, vec!["test"]);
        }
    }

    mod one_section {
        use super::*;

        #[crate::test(rye_path = "crate")]
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
            assert_eq!(history, vec!["setup", "section1", "teardown"]);
        }
    }

    mod multi_section {
        use super::*;

        #[crate::test(rye_path = "crate")]
        fn test_case() {
            HISTORY.with(|history| history.borrow_mut().push("setup"));

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
                    "setup", "section1", "teardown", //
                    // phase 2
                    "setup", "section2", "teardown",
                ]
            );
        }
    }

    mod nested_section {
        use super::*;

        #[crate::test(rye_path = "crate")]
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
                    "setup",
                    "section1:setup",
                    "section2",
                    "section1:teardown",
                    "test",
                    "teardown",
                    // phase 2
                    "setup",
                    "section1:setup",
                    "section3",
                    "section1:teardown",
                    "test",
                    "teardown",
                    // phase 3
                    "setup",
                    "test",
                    "section4",
                    "teardown",
                ]
            );
        }
    }

    mod smoke_async {
        use super::*;

        #[crate::test(rye_path = "crate")]
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
                    "setup",
                    "section1:setup",
                    "section2",
                    "section1:teardown",
                    "test",
                    "teardown",
                    // phase 2
                    "setup",
                    "section1:setup",
                    "section3",
                    "section1:teardown",
                    "test",
                    "teardown",
                    // phase 3
                    "setup",
                    "test",
                    "section4",
                    "teardown",
                ]
            );
        }
    }
}
