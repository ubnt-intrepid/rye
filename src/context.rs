use crate::section::Section;
use std::{cell::Cell, marker::PhantomData, mem, ptr::NonNull};

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<TestContext<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<TestContext<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| tls.set(self.0.take()));
    }
}

pub(crate) struct TestContext<'a> {
    section: &'a Section,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> TestContext<'a> {
    pub(crate) fn new(section: &'a Section) -> Self {
        Self {
            section,
            _marker: PhantomData,
        }
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

    pub(crate) fn section(&self) -> &Section {
        &*self.section
    }
}

#[derive(Debug)]
pub(crate) struct AccessError {
    _p: (),
}

#[cfg(feature = "futures")]
mod futures {
    use super::*;
    use futures_core::{
        future::Future,
        task::{self, Poll},
    };
    use pin_project::pin_project;
    use std::pin::Pin;

    impl TestContext<'_> {
        pub(crate) async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
        where
            Fut: Future,
        {
            ScopeAsync { fut, ctx: self }.await
        }
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
}
