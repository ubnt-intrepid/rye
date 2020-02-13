use crate::test_case::{SectionId, TestDesc};
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{
    channel::oneshot,
    executor::ThreadPool,
    future::Future,
    task::SpawnExt as _,
    task::{self, Poll},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use mimicaw::{Outcome, Test};
use pin_project::pin_project;
use std::{
    cell::Cell, fmt::Write as _, mem, panic::AssertUnwindSafe, pin::Pin, ptr::NonNull, sync::Once,
};

pub struct TestSuite<'a> {
    test_cases: &'a mut Vec<Test<TestData>>,
}

impl TestSuite<'_> {
    #[doc(hidden)] // private API
    pub fn register<F>(&mut self, desc: TestDesc, test_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        let ignored = desc.ignored;
        self.test_cases.push(
            Test::test(
                desc.name,
                TestData {
                    desc,
                    test_fn: TestFn::Sync(Box::new(test_fn)),
                },
            )
            .ignore(ignored),
        );
    }

    #[doc(hidden)] // private API
    pub fn register_async<F, Fut>(&mut self, desc: TestDesc, test_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let ignored = desc.ignored;
        self.test_cases.push(
            Test::test(
                desc.name,
                TestData {
                    desc,
                    test_fn: TestFn::Async(Box::new(move || Box::pin(test_fn()))),
                },
            )
            .ignore(ignored),
        );
    }
}

pub fn run_tests(tests: &[&dyn Fn(&mut TestSuite<'_>)]) {
    let args = mimicaw::Args::from_env().unwrap_or_else(|st| st.exit());

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        maybe_unwind::set_hook();
    });

    let mut test_cases = vec![];
    for &test in tests {
        test(&mut TestSuite {
            test_cases: &mut test_cases,
        });
    }

    let mut pool = ThreadPool::new().unwrap();

    let st = futures::executor::block_on(mimicaw::run_tests(
        &args,
        test_cases,
        |_desc, data: TestData| data.run(&mut pool),
    ));
    st.exit();
}

struct TestData {
    desc: TestDesc,
    test_fn: TestFn,
}

enum TestFn {
    Sync(Box<dyn Fn() + Send + 'static>),
    Async(
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>,
    ),
}

impl TestData {
    fn run(self, pool: &mut ThreadPool) -> Pin<Box<dyn Future<Output = Outcome> + Send + 'static>> {
        let Self { desc, test_fn } = self;
        match test_fn {
            TestFn::Sync(f) => {
                let (tx, rx) = oneshot::channel();
                std::thread::spawn(move || {
                    let res = expected(|| {
                        maybe_unwind(AssertUnwindSafe(|| {
                            if desc.leaf_sections.is_empty() {
                                TestContext::new(&desc, None).scope(&f);
                            } else {
                                for &section in desc.leaf_sections {
                                    TestContext::new(&desc, Some(section)).scope(&f);
                                }
                            }
                        }))
                    });
                    let _ = tx.send(res);
                });

                Box::pin(async move {
                    match rx.await {
                        Ok(res) => make_outcome(res),
                        Err(rx_err) => {
                            Outcome::failed().error_message(format!("unknown error: {}", rx_err))
                        }
                    }
                })
            }
            TestFn::Async(f) => {
                let handle = pool
                    .spawn_with_handle(async move {
                        if desc.leaf_sections.is_empty() {
                            TestContext::new(&desc, None).scope_async(f()).await;
                        } else {
                            for &section in desc.leaf_sections {
                                TestContext::new(&desc, Some(section))
                                    .scope_async(f())
                                    .await;
                            }
                        }
                    })
                    .unwrap();
                Box::pin(async move {
                    let res = AssertUnwindSafe(handle).maybe_unwind().expected().await;
                    make_outcome(res)
                })
            }
        }
    }
}

fn make_outcome(res: (Result<(), Unwind>, Option<Disappoints>)) -> Outcome {
    match res {
        (Ok(()), None) => Outcome::passed(),
        (Ok(()), Some(disappoints)) => Outcome::failed().error_message(disappoints.to_string()),
        (Err(unwind), disappoints) => {
            let mut msg = String::new();
            let _ = writeln!(&mut msg, "{}", unwind);
            if let Some(disappoints) = disappoints {
                let _ = writeln!(&mut msg, "{}", disappoints);
            }
            Outcome::failed().error_message(msg)
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
    pub(crate) fn new(desc: &'a TestDesc, section: Option<SectionId>) -> Self {
        Self { desc, section }
    }

    pub(crate) fn scope<F, R>(&mut self, f: F) -> R
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

    pub(crate) async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
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
        self.section.map_or(false, |section| {
            let section = self
                .desc
                .sections
                .get(&section)
                .expect("invalid section id is set");
            section.ancestors.contains(&id)
        })
    }
}

#[derive(Debug)]
pub(crate) struct AccessError {
    _p: (),
}

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
