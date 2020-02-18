use crate::{
    report::Outcome,
    test::{SectionId, Test, TestDesc, TestFn},
};
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{
    executor::ThreadPool,
    future::{BoxFuture, Future},
    task::SpawnExt as _,
    task::{self, Poll},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use pin_project::pin_project;
use std::{cell::Cell, io, mem, panic::AssertUnwindSafe, pin::Pin, ptr::NonNull};

pub trait TestExecutor {
    type Handle: Future<Output = Outcome>;

    fn execute<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = Outcome> + Send + 'static;
    fn execute_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> Outcome + Send + 'static;
}

pub struct DefaultTestExecutor {
    pool: ThreadPool,
}

impl DefaultTestExecutor {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            pool: ThreadPool::new()?,
        })
    }
}

impl TestExecutor for DefaultTestExecutor {
    type Handle = BoxFuture<'static, Outcome>;

    fn execute<Fut>(&mut self, fut: Fut) -> Self::Handle
    where
        Fut: Future<Output = Outcome> + Send + 'static,
    {
        let handle = self.pool.spawn_with_handle(fut);
        Box::pin(async move {
            match handle {
                Ok(handle) => handle.await,
                Err(spawn_err) => {
                    Outcome::failed().error_message(format!("unknown error: {}", spawn_err))
                }
            }
        })
    }

    fn execute_blocking<F>(&mut self, f: F) -> Self::Handle
    where
        F: FnOnce() -> Outcome + Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|rx_err| {
                Outcome::failed().error_message(format!("unknown error: {}", rx_err))
            })
        })
    }
}

pub(crate) fn start_test<E: ?Sized>(test_case: &Test, executor: &mut E) -> E::Handle
where
    E: TestExecutor,
{
    let desc = test_case.desc.clone();
    match test_case.test_fn {
        TestFn::SyncTest(f) => executor.execute_blocking(move || {
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
            make_outcome(res)
        }),
        TestFn::AsyncTest(f) => executor.execute(async move {
            let res = AssertUnwindSafe(async move {
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
            .maybe_unwind()
            .expected()
            .await;
            make_outcome(res)
        }),
    }
}

fn make_outcome(res: (Result<(), Unwind>, Option<Disappoints>)) -> Outcome {
    match res {
        (Ok(()), None) => Outcome::passed(),
        (Ok(()), Some(disappoints)) => Outcome::failed().error_message(disappoints.to_string()),
        (Err(unwind), disappoints) => {
            use std::fmt::Write as _;
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
